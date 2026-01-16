using System;
using System.ComponentModel.DataAnnotations;

namespace PondoBro.Backend.Models;

public class Transaction
{
    public int Id { get; set; }

    [Required]
    public DateTime Date { get; set; } = DateTime.UtcNow;

    [Required]
    public string Description { get; set; } = string.Empty;

    public string Category { get; set; } = string.Empty;

    public long Amount { get; set; }

    public int? UserId { get; set; }

    public User? User { get; set; }
}
