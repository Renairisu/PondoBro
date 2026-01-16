using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using PondoBro.Backend.Data;
using PondoBro.Backend.Models;
using PondoBro.Backend.Services;

namespace PondoBro.Backend.Controllers;

[ApiController]
[Route("api/auth")]
public class AuthController : ControllerBase
{
    private readonly AppDbContext _db;
    private readonly IJwtService _jwt;

    public AuthController(AppDbContext db, IJwtService jwt)
    {
        _db = db;
        _jwt = jwt;
    }

    [HttpPost("register")]
    public async Task<IActionResult> Register(RegisterRequest request)
    {
        if (!ModelState.IsValid) return ValidationProblem(ModelState);

        try
        {
            var exists = await _db.Users.AnyAsync(u => u.Email == request.Email);
            if (exists) return Conflict(new { error = "Email already exists" });

            var user = new User
            {
                Email = request.Email,
                PasswordHash = BCrypt.Net.BCrypt.HashPassword(request.Password)
            };

            _db.Users.Add(user);
            await _db.SaveChangesAsync();

            await IssueSessionCookie(user);
            return Ok(BuildAuthResponse(user));
        }
        catch (DbUpdateException dbEx)
        {
            var inner = dbEx.InnerException?.Message ?? string.Empty;
            if (inner.Contains("UNIQUE", StringComparison.OrdinalIgnoreCase) || inner.Contains("constraint", StringComparison.OrdinalIgnoreCase))
            {
                return Conflict(new { error = "Email already exists" });
            }

            return Problem("An unexpected database error occurred while creating the account.");
        }
        catch (Exception)
        {
            return Problem("An unexpected error occurred while creating the account.");
        }
    }

    [HttpPost("login")]
    public async Task<IActionResult> Login(LoginRequest request)
    {
        if (!ModelState.IsValid) return ValidationProblem(ModelState);

        try
        {
            var user = await _db.Users.FirstOrDefaultAsync(u => u.Email == request.Email);
            if (user is null || !BCrypt.Net.BCrypt.Verify(request.Password, user.PasswordHash))
            {
                return Unauthorized(new { error = "Invalid email or password" });
            }

            await IssueSessionCookie(user);
            return Ok(BuildAuthResponse(user));
        }
        catch (Exception)
        {
            return Problem("An unexpected error occurred while attempting to log in.");
        }
    }

    [HttpPost("refresh")]
    public async Task<IActionResult> Refresh()
    {
        if (!Request.Cookies.TryGetValue("refresh_token", out var token))
        {
            return Unauthorized(new { error = "Missing refresh token" });
        }

        try
        {
            var session = await _db.Sessions
                .Include(s => s.User)
                .FirstOrDefaultAsync(s => s.RefreshToken == token);

            if (session is null || session.ExpiresAt < DateTime.UtcNow || session.User is null)
            {
                return Unauthorized(new { error = "Invalid or expired refresh token" });
            }

            return Ok(BuildAuthResponse(session.User));
        }
        catch (Exception)
        {
            return Problem("An unexpected error occurred while refreshing the session.");
        }
    }

    [HttpPost("logout")]
    public async Task<IActionResult> Logout()
    {
        try
        {
            if (Request.Cookies.TryGetValue("refresh_token", out var token))
            {
                var session = await _db.Sessions.FirstOrDefaultAsync(s => s.RefreshToken == token);
                if (session is not null)
                {
                    _db.Sessions.Remove(session);
                    await _db.SaveChangesAsync();
                }
            }

            Response.Cookies.Append("refresh_token", "", new CookieOptions
            {
                HttpOnly = true,
                Secure = false,
                SameSite = SameSiteMode.None,
                Expires = DateTime.UtcNow.AddDays(-1)
            });

            return Ok(new { ok = true });
        }
        catch (Exception)
        {
            return Problem("An unexpected error occurred while logging out.");
        }
    }

    private object BuildAuthResponse(User user)
    {
        var access = _jwt.CreateAccessToken(user);
        return new
        {
            access_token = access,
            token_type = "Bearer",
            expires_in = 60 * 60,
            user = new { user.Id, user.Email, user.Role }
        };
    }

    private async Task IssueSessionCookie(User user)
    {
        var refreshToken = _jwt.CreateRefreshToken();
        var session = new Session
        {
            UserId = user.Id,
            RefreshToken = refreshToken,
            ExpiresAt = _jwt.GetRefreshExpiry()
        };

        _db.Sessions.Add(session);
        await _db.SaveChangesAsync();

        Response.Cookies.Append("refresh_token", refreshToken, new CookieOptions
        {
            HttpOnly = true,
            Secure = false,
            SameSite = SameSiteMode.None,
            Expires = session.ExpiresAt
        });
    }
}

public class RegisterRequest
{
    [System.ComponentModel.DataAnnotations.Required]
    [System.ComponentModel.DataAnnotations.EmailAddress]
    public string Email { get; set; } = string.Empty;

    [System.ComponentModel.DataAnnotations.Required]
    [System.ComponentModel.DataAnnotations.MinLength(8)]
    public string Password { get; set; } = string.Empty;

    [System.ComponentModel.DataAnnotations.Required]
    [System.ComponentModel.DataAnnotations.Compare(nameof(Password))]
    public string ConfirmPassword { get; set; } = string.Empty;
}

public class LoginRequest
{
    [System.ComponentModel.DataAnnotations.Required]
    [System.ComponentModel.DataAnnotations.EmailAddress]
    public string Email { get; set; } = string.Empty;

    [System.ComponentModel.DataAnnotations.Required]
    [System.ComponentModel.DataAnnotations.MinLength(8)]
    public string Password { get; set; } = string.Empty;
}
