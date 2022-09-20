# Functions in this repo

## SubmitClaim

`POST` `/api/submitclaim`

This action lets a user try and submit an airdrop claim. This will 1. associate one or more addresses with a secret network address and 2. validate the ownership of
said addresses. To do this, a user must create a valid permit that is signed with their Keplr wallet. For now the data is mocked and not tested with real data, and only the address secret1p0vgghl8rw4ukzm7geyy0f0tl29glxrtnlalue will return airdrop data.

### Request

#### Body Params

| param        | type              | function                           |
|--------------|-------------------|------------------------------------|
| request.body | Claims            | User submitted claims              |
| address      | string            | Address that signed the permit     |
| permit       | Permit (secretjs) | Permit that was signed by the user |

```typescript
interface Claims {
    claims: Claim[]
}

interface Claim {
    address: string,
    permit: Permit
}

```

#### Example

```bash
curl -X POST 'https://legendao-testnet.azurewebsites.net/api/submitclaim?code=ZoRJruWZzazk9YxmSJUO5Hg2va/Aa0gyM37soFB5eh8VKLBv2dzSjQ=='   -H 'content-type: application/json'  -d '{"claims": [{"permit": {"params":{"chain_id":"secret-4","permit_name":"default","allowed_tokens":["secret1asdf"],"permissions":["owner"]},"signature":{"pub_key":{"type":"tendermint/PubKeySecp256k1","value":"AgyShSTNVC3olnm/VAPUvrN5IbGrqe1oH+E5/H3F9SUB"},"signature":"K0/aw2D8tx3JVYgddPdKMoQKLZDdYKQvU/wjvMUiihokGxLzqjKQpxkKNAtj1kTl0wovpXDxZuSi1ykkR8Sk8Q=="}}, "address": "secret1p0vgghl8rw4ukzm7geyy0f0tl29glxrtnlalue"}]}'
```

### Response

| status | type                             | function                                                                                                                              |
|--------|----------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|
| 200    | `body: { claimed_for: string[]}` | Request succesful. Returns a list of addresses that have been approved and are now pending airdrop claim                              |
| 400    | `body: { error: string }`        | Error claiming airdrop. Reasons include: failed to validate claim, users not in airdrop whitelist, users have already claimed airdrop |

## SendDistributeTx

This is a timed function that scans the DB for new claims and sends them in a batch to the airdrop contract.
The airdrop contract will send the funds to the corresponding addresses. It does not change the status of the claim, and will simply attempt to approve
all current "Submitted" claims every time (the contract will only allow each address to claim once and ignore duplicates)

## MonitorSuccess

This is a timed function that scans the DB for submitted claims, and checks the chain if they were successful yet or not.
If they are, it changes the status to "Claimed". If not, the status will remain in "Submitted"

## CheckStatus

`GET` `/api/checkstatus`

This function allows the user to check the status of his claim.

### Request

#### Query Params

| param     | type     | function                                                 |
|-----------|----------|----------------------------------------------------------|
| addresses | string[] | Bech32 addresses in CSV format to check claim status for |

#### Example

```bash
curl 'https://legendao-testnet.azurewebsites.net/api/checkstatus?code=/ebdKLfLEklqdYN0WXHqLapwlKxp8mqP2s2bZBhTMxFd28mnhWsYCQ==&addresses=secret1p0vgghl8rw4ukzm7geyy0f0tl29glxrtnlalue'
```

### Response

| status | type                                          | function                                                                                                  |
|--------|-----------------------------------------------|-----------------------------------------------------------------------------------------------------------|
| 200    | `body: { status: {[address]: ClaimResponse}}` | Request successful. Returns a list of addresses that have been approved and are now pending airdrop claim |
| 400    | `body: { error: string }`                     | Error querying status                                                                                     |

```typescript

type CLAIM_STATUS = "NotClaimed" | "Claimed" | "Submitted" | "NotWhitelisted";

interface ClaimResponse {
    status: CLAIM_STATUS,
    amount: number
}
```

### Statuses

| status         | type                                                                              |
|----------------|-----------------------------------------------------------------------------------|
| NotClaimed     | User is in the whitelist but has not yet claimed his airdrop                      |
| Claimed        | User is in the whitelist and has already claimed his airdrop                      |
| NotWhitelisted | User is not in the whitelist                                                      |
| Submitted      | User is in the whitelist, has submitted his claim and airdrop delivery is pending |


## ModifyAirdropData (TEST ONLY)

Testing function that allows inserting or modification of claim data

### Example usage - resetting claim status for a user

```bash
curl 'https://legendao-testnet.azurewebsites.net/api/modifyairdropdata?code=XYfJRy9s1VWAYrnaDJNRzB01TraWXgKOLGMA3oRA8/ppwpaILOufog==&address=secret1p0vgghl8rw4ukzm7geyy0f0tl29glxrtnlalue&status=NotClaimed&delete_permit=true'
```

### Example usage - adding new user

```bash
curl 'https://legendao-testnet.azurewebsites.net/api/modifyairdropdata?code=XYfJRy9s1VWAYrnaDJNRzB01TraWXgKOLGMA3oRA8/ppwpaILOufog==&address=random_new_user&status=NotClaimed&amount=1000'
```