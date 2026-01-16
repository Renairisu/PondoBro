using System.ComponentModel.DataAnnotations;

namespace PondoBro.Backend.Models;

public class User
{
    public int Id { get; set; }

    [Required, EmailAddress]
    public string Email { get; set; } = string.Empty;

    [Required]
    public string PasswordHash { get; set; } = string.Empty;

    [Required]
    public string Role { get; set; } = "User";

    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
    public List<Session> Sessions { get; set; } = new();
    public List<Transaction> Transactions { get; set; } = new();
}
