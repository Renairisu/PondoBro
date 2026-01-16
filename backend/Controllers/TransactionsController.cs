using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using PondoBro.Backend.Data;
using PondoBro.Backend.Models;

namespace PondoBro.Backend.Controllers;

[ApiController]
[Route("api/transactions")]
public class TransactionsController : ControllerBase
{
    private readonly AppDbContext _db;

    public TransactionsController(AppDbContext db)
    {
        _db = db;
    }

    [HttpGet]
    public async Task<IActionResult> GetAll()
    {
        int? userId = null;

        // Prefer session cookie if present
        if (Request.Cookies.TryGetValue("refresh_token", out var token))
        {
            var session = await _db.Sessions.FirstOrDefaultAsync(s => s.RefreshToken == token);
            if (session is not null) userId = session.UserId;
        }

        // Fallback to JWT bearer token if available
        if (userId is null && User?.Identity?.IsAuthenticated == true)
        {
            var sub = User.FindFirst(System.IdentityModel.Tokens.Jwt.JwtRegisteredClaimNames.Sub)?.Value
                      ?? User.FindFirst(System.Security.Claims.ClaimTypes.NameIdentifier)?.Value;
            if (int.TryParse(sub, out var parsed)) userId = parsed;
        }

        if (userId is null) return Unauthorized(new { error = "Not authenticated" });

        var list = await _db.Transactions
            .Where(t => t.UserId == userId.Value)
            .OrderByDescending(t => t.Date)
            .ToListAsync();

        return Ok(list);
    }

    public class CreateTransactionRequest
    {
        public string? Date { get; set; }
        public string Description { get; set; } = string.Empty;
        public string Category { get; set; } = string.Empty;
        public long Amount { get; set; }
    }

    [HttpPost]
    public async Task<IActionResult> Create(CreateTransactionRequest req)
    {
        try
        {
            int? userId = null;

            if (Request.Cookies.TryGetValue("refresh_token", out var token))
            {
                var session = await _db.Sessions.FirstOrDefaultAsync(s => s.RefreshToken == token);
                if (session is not null) userId = session.UserId;
            }

            if (userId is null && User?.Identity?.IsAuthenticated == true)
            {
                var sub = User.FindFirst(System.IdentityModel.Tokens.Jwt.JwtRegisteredClaimNames.Sub)?.Value
                          ?? User.FindFirst(System.Security.Claims.ClaimTypes.NameIdentifier)?.Value;
                if (int.TryParse(sub, out var parsed)) userId = parsed;
            }

            if (userId is null) return Unauthorized(new { error = "Not authenticated" });

            var tx = new Transaction
            {
                Date = string.IsNullOrWhiteSpace(req.Date) ? DateTime.UtcNow : DateTime.Parse(req.Date),
                Description = req.Description ?? string.Empty,
                Category = req.Category ?? string.Empty,
                Amount = req.Amount,
                UserId = userId.Value
            };

            _db.Transactions.Add(tx);
            await _db.SaveChangesAsync();

            return CreatedAtAction(nameof(GetAll), new { id = tx.Id }, tx);
        }
        catch (Exception)
        {
            return Problem("Could not create transaction.");
        }
    }
}
