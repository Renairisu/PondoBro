using System.ComponentModel.DataAnnotations;

namespace PondoBro.Backend.Models;

public class Session
{
    public int Id { get; set; }

    [Required]
    public string RefreshToken { get; set; } = string.Empty;

    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;

    public DateTime ExpiresAt { get; set; }

    public int UserId { get; set; }

    public User? User { get; set; }
}
