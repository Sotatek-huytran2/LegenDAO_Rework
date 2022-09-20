# Functions in this repo

## CreationForm

`POST` `/api/creationform`

This action submits a creation form from the UI, stores it in the database and submits a notification about the new application

### Request

#### Body Params

| param   | type   | function                                                           |
|---------|--------|--------------------------------------------------------------------|
| name    | string | Corresponds to the name field in the creation form                 |
| email   | string | Corresponds to the email field in the creation form                |
| title   | string | Corresponds to the "What do you create" field in the creation form |
| details | string | Corresponds to the "Anything else..." field in the creation form   |

```typescript
interface CreationForm {
    name: string,
    email: string,
    title: string,
    details: string
}


```

#### Example

```bash
curl -X POST 'https://legendao-testnet.azurewebsites.net/api/creationform'   -H 'content-type: application/json'  -d '{"name": "asdf", "email": "asdf", "title": "asdf", "details": "asfd"}'
```

### Response

| status | type                      | function                                                                                       |
|--------|---------------------------|------------------------------------------------------------------------------------------------|
| 200    | ``                        | Request succesful                                                                              |
| 400    | `body: { error: string }` | Error if email is malformed, content is non-alphanumeric or content length limits are reached  |
| 500    | ``                        | Internal error when handling form. No can be taken by the user except retrying at a later time |


## TokenData

`GET` `/api/tokendata`

Get LGND token data from the API endpoint

### Request

#### Body Params

| param   | type   | function                                                           |
|---------|--------|--------------------------------------------------------------------|

#### Example

```bash
curl 'https://legendao-testnet.azurewebsites.net/api/tokendata'
```

### Response

| status | type        | function                                                                                       |
|--------|-------------|------------------------------------------------------------------------------------------------|
| 200    | `TokenData` | Request succesful                                                                              |

```typescript
interface TokenData {
    price_usd: number,
    apy: number,
    liquidity: number,
    daily_volume: number
}
```

#### Example

```json
{
  "price_usd": 6.3,
  "apy": 85,
  "liquidity": 50000000,
  "daily_volume": 5000000
}
```