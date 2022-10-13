const CryptoJS = require('crypto-js');
const SHA256 = require('crypto-js/sha256');
const secp256k1 = require('secp256k1');
const { RawKey, LCDClient } = require('@terra-money/terra.js');

function decrypt(transitmessage, pass) {
    const salt = CryptoJS.enc.Hex.parse(transitmessage.substr(0, 32));
    const iv = CryptoJS.enc.Hex.parse(transitmessage.substr(32, 32));
    const encrypted = transitmessage.substring(64);

    const keySize = 256;
    const iterations = 100;
    const key = CryptoJS.PBKDF2(pass, salt, {
        keySize: keySize / 32,
        iterations: iterations,
    });

    return CryptoJS.AES.decrypt(encrypted, key, {
        iv: iv,
        padding: CryptoJS.pad.Pkcs7,
        mode: CryptoJS.mode.CBC,
    }).toString(CryptoJS.enc.Utf8);
}

const main = async () => {

    const terra = new LCDClient({
        chainID: "columbus-5",
        URL: "https://lcd.terra.dev",
    });

    // Import Private key 
    const exportedWallet = JSON.parse(
        Buffer.from(
            "eyJuYW1lIjoibWFtYm8iLCJhZGRyZXNzIjoidGVycmExdW1wcGNubnNkd2E2bTZyaHByYWF6cnV6ZmdlZWhtaGZ1dXdqZnUiLCJlbmNyeXB0ZWRfa2V5IjoiZTUwMTlhNDNkOTNmY2UyMjE2ZjkwYzBlNzBjNWJjNjUyYWVkMzlhNDg1ZGUzN2ZjOTg3ZTFkZDU3M2NhODFmNUVCdXRVSEtrS2VhSDVRU3dkZGJvWmZoQnNpN3ZlclR1VitVVkEwVk44NjhFbXN1M25YbHNJV1lPRFR0dXVKZE8xR1FSaVZNSmltSXlVQ2lRajJsa3VleW5SRXk4a1J0akgwUkxKRmNzcHdnPSJ9",
            // "eyJuYW1lIjoiemlwcG8iLCJhZGRyZXNzIjoidGVycmExbjJ1bWhtM3NzOHIwc2pwcGQ1ZDVjeDJkdDRlYXhzdjlxdXluN3UiLCJlbmNyeXB0ZWRfa2V5IjoiODg4MTkwZjZlZWU2MzQ1Yzg0ZGUwYWJiN2M1NjE2MDYwOTdjODc4NTJiNDBlMThmYmRlOGZjMWQxYjUxOTg5NU5ML25BeElrRWIxbjNpRkpSM3ljVTRuN2lib2Y5VTFJcWE1dk5uaTUxTk9jTWxGNVZIRU1YbHBuVEhsUnVnMWk2L2FrNFNvN2FDTVA5L0lSTi9oUjhCZ0ovZXo1RjhrRHJqcjVXWFh1WnJNPSJ9",
            "base64"
        ).toString("utf8")
    );



    const decryptedKey = decrypt(exportedWallet.encrypted_key, "@Solid0209");
    
    const rawKey = new RawKey(Buffer.from(decryptedKey, "hex"));


    const minting_grant = { 
        "minting_granted": { 
            "recipient": "buyer", 
            "quantity": "2", 
            "nonce": "0" 
        } 
    }


    // Sign minting instruction by the imported private key
    const signature = await rawKey.sign(Buffer.from(JSON.stringify(minting_grant)));


    // Verify valid public key if needed
    const verified = secp256k1.publicKeyVerify(
        new Uint8Array(Buffer.from(rawKey.publicKey.key.toString('base64'), 'base64'))
    )

    console.log("VERIFIED: ", verified);

    // Verify instruction off-chain 
    const bool = secp256k1.ecdsaVerify(
        new Uint8Array(Buffer.from(signature)),
        new Uint8Array(Buffer.from(SHA256(minting_grant).toString(), 'hex')),
        new Uint8Array(Buffer.from(rawKey.publicKey.key.toString('base64'), 'base64'))
    )
    console.log("Sign by private key: ", bool);


    // console.log(signature.toString("base64"));
    // console.log(rawKey.publicKey.key);
    // console.log(rawKey.publicKey.key.toString("base64"));
    // console.log(Buffer.from({"minting_granted":{"recipient":"buyer","quantity":"2","nonce":"0"}}).toString("base64"));
}

main();