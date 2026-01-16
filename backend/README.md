# PondoBro C# Backend (SQLite)

## Requirements

- .NET 8 SDK

## Setup

```powershell
cd backend-csharp
```

Apply migrations (first time):

```powershell
dotnet tool install --global dotnet-ef
$env:ASPNETCORE_ENVIRONMENT = "Development"
dotnet ef migrations add InitialCreate

dotnet ef database update
```

## Run

```powershell
dotnet run
```

## Endpoints

- GET /api/health
- POST /api/auth/register
- POST /api/auth/login (sets HttpOnly refresh_token cookie)

## Notes

- Update Jwt:Secret in appsettings.json before production.
- SQLite DB file is stored at backend-csharp/data/pondobro.db
