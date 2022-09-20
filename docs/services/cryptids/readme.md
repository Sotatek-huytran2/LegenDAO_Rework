## Functions in this repo

### IsWhitelisted

`GET` `/api/IsWhitelisted?address={}`

Returns whether the user is whitelisted for this mint.
Does not return the amount of tokens available to him

#### Request

##### Query Params

| param   | type   | function                                     | required |
|---------|--------|----------------------------------------------|----------|
| address | string | Bech32 address to check whitelist status for | true     |

##### Example

```bash
curl 'https://cryptids-testnet.azurewebsites.net/api/iswhitelisted?address=secret1p0vgghl8rw4ukzm
7geyy0f0tl29glxrtnlalue&code=zia7kDoux1zpLWsflavaXR/mfI9rTJjQRmRiJ9bPvBZOwNlqIaOvmQ=='
```


##### Response

| status | type                           | function                                                           |
|--------|--------------------------------|--------------------------------------------------------------------|
| 200    | `body: { whitelist: boolean }` | Request successful. Returns whether or not the user is whitelisted |
| 400    | `body: { error: string }`      | Error querying status                                              |

##### Example

```json
{
  "whitelist": true
}
```

### AttributeStatistics

`GET` `/api/AttributeStatistics?...`

Returns the rarity of attributes. Supports querying one of each type of attribute.
Returns their score and rarity

#### Request
 
##### Query Params

| param      | type   | function                  | required |
|------------|--------|---------------------------|----------|
| background | string | Background attribute type | false    |
| body       | string | body attribute type       | false    |
| hairs      | string | hairs attribute type      | false    |
| eyes       | string | eyes attribute type       | false    |
| rhand      | string | Right Hand attribute type | false    |
| lhand      | string | Left Hand attribute type  | false    |
| wear       | string | Wear attribute type       | false    |
| horns      | string | Horns attribute type      | false    |

##### Example

```bash
curl -X GET 'https://cryptids-testnet.azurewebsites.net/api/attributestatistics?background=white&lhand=none&code=p5CbBJPQfYHjNq/PxjAEAsebbVjOgP2h2qkk8zQyvd1KDxhLynQgeg=='
```

#### Response

| status | type                                                                    | function                                                                                                            |
|--------|-------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------|
| 200    | `body: { attributes: [attribute]: {score: float, percentage: float } }` | Request successful. For each requested attribute will return their corresponding score and percentage of appearance |
| 400    | `body: { error: string }`                                               | Error                                                                                                               |


##### Example
```json
{
  "attributes": {
    "background": {
      "white": {
        "score": "26",
        "percentage": "50"
      }
    },
    "lhand": {
      "none": {
        "score": "63",
        "percentage": "51"
      }
    }
  }
}
```

