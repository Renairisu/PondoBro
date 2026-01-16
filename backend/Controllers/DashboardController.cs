using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using PondoBro.Backend.Data;

namespace PondoBro.Backend.Controllers;

[ApiController]
[Route("api/dashboard")]
public class DashboardController : ControllerBase
{
    private readonly AppDbContext _db;

    public DashboardController(AppDbContext db)
    {
        _db = db;
    }

    [HttpGet("summary")]
    public async Task<IActionResult> Summary()
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

        var totalIncome = await _db.Transactions.Where(t => t.UserId == userId && t.Amount > 0).SumAsync(t => (long?)t.Amount) ?? 0L;
        var totalExpenses = await _db.Transactions.Where(t => t.UserId == userId && t.Amount < 0).SumAsync(t => (long?)t.Amount) ?? 0L;
        totalExpenses = Math.Abs(totalExpenses);

        var balance = totalIncome - totalExpenses;

        return Ok(new { total_income = totalIncome, total_expenses = totalExpenses, balance });
    }
}
