# API-Reference

## Basis-URL
- **Dev**: `http://localhost:3000/acm`
- **Prod**: `http://<domain>/acm`

## Endpoints

### Healthcheck
```http
GET /acm
→ 200 {"status":"ok","message":"ACM API Connection Monitor"}
```

### Authentication
```http
POST /acm/login
Body: {"email":"test@test.de","password":"123"}
→ 200 {"userid":1,"emailadress":"test@test.de"}
→ 401 {"error":"Invalid email or password"}

POST /acm/createAccount
Body: {"email":"test@test.de","password":"123"}
→ 200 {"userid":1,"emailadress":"test@test.de"}
→ 409 {"error":"Email already exists"}
```

### User-Endpoints
```http
GET /acm/home?id=1
→ 200 [{"endpointid":1,"url":"...","status":true,...}]

GET /acm/user?id=1
→ 200 {"userid":1,"emailadress":"test@test.de","password":"$2b$12..."}
```

### User-Verwaltung
```http
PUT /acm/user/changePassword
Body: {"userid":1,"old_password":"alt","new_password":"neu"}
→ 200 {"status":"ok"}

PUT /acm/user/changeEmail
Body: {"userid":1,"new_email":"neu@test.de"}
→ 200 {"status":"ok"}

DELETE /acm/user/deleteAccount
Body: {"userid":1}
→ 200 {"status":"ok"}
```

### Endpoint-CRUD
```http
PUT /acm/addEndpoint
Body: {"userid":1,"url":"https://api.example.com"}
→ 200 {"endpointid":1}

PUT /acm/updateEndpoint
Body: {"endpointid":1,"url":"https://neu.example.com"}
→ 200 {"status":"ok"}

PUT /acm/deleteConfirm
Body: {"endpointid":1}
→ 200 {"status":"ok"}
```

### Monitoring
```http
PUT /acm/setIntervall
Body: {"endpointid":1,"seconds":60}
→ 200 {"status":"ok"}

GET /acm/log?id=1
→ 200 [{"endpointid":1,"status":true,"statusdate":"2026-05-20","statustime":"14:30:00"}]
```
