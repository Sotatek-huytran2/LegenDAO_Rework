Components

1. Airdrop

1.1 Snapshot data - from when?
	* Networks: Cosmos, Terra, Osmo  - we have nodes on all these networks. 
    Can stop the node and export genesis for easy data fetching. Need to make sure nodes are still active
	
	* Open questions - Limits on airdrop? Amounts? 
	
	
1.2. Airdrop Contract

	* Airdrop to multiple networks
	* requires modification of permits to support multiple hrps
	* Semi-Offline airdrop: Contract to post your permit, then a service validates once every couple of minutes

1.3. Airdrop UI

	* Done as a part of the main UI?
	* Will support Terrastation or just Keplr?
	
2. Snip20 Token - LGND

	* Standard snip24 token + minting
	
3. IBC Token - wLGND

	* Mint IBC tokens from new chain and transfer them over
	* Deploy sscrt contract for the new pair
	* Wrap/unwrap from to IBC token needs to be in the UI - (maybe make it transparent by doing both the convert and the ibc send in 1 action?)

4. "Platform"

4.1 Manager Contract

* Aquired tokens are to be locked inside the "app" for 21 days. This means tokens will be minted to a contract and the user
will be credited with SPY tokens. 

* "Withdrawing" from the Manager triggers a 21 day waiting period. Ideally we'd want the claim to be automatic, although manual "claiming" will still be necessary as a fallback

* SPY tokens can be used for specific things:

	* Buying an NFT
	* Staking
	* Buying items (tbd) <- this means we need to have a way to add future actions as well. Probably can just use a "send"
	
4.2 Staking Contract

4.2.1 Need to do "delegated staking", whereby the "manager" contract will deposit for the user - this makes tracking balances really annoying, I know.
4.2.2 Inflation rate can be set manually	

4.3 NFT Mint/purchasing contract

4.3.1 Support for whitelist spots
4.3.2 Support for delegated purchasing (i.e. mint for someone else)
4.3.3 Purchasing contract per NFT (simpler to copy-paste the contract)


5. NFT Token - Yeti

* Public Image/attributes only
* Art + randomization will not be done by us

6. NFT Token - Generic

6.1 Private/Public image and text attributes only
6.2 Use pinata or something for this
6.3 Set up scripts to deploy/set up automatically so we don't have to manually handle it every time

7. Vesting Contract

7.1 Contract that receives a <amount, deadline> and allows you to unlock an amount every <x> time (all manual)

* Questions - is the vesting UI part of the main page? Do we have to build it separately?

8. Token Store

8.1 This contract will allow a user to send a native token and purchase LGND. 
8.2 The price will be fed manually via a script (calculated externally). 
8.3 If the contract detects price not being updated, it will turn off after <X> time
8.4 The price will be calculated vs. the price in Osmosis
8.5 Supported currencies:
	a. UST
	b. LUNA
	c. ATOM
	d. OSMO
	e. SCRT

* Open Questions - 
	What are the limits of this mechanism? Max total amount that can be sold?

9. Missions

* Ignoring this for now

12. Ethereum or BSC wrapped LGND token?
