# TORVPN BUILD FIX INSTRUCTIONS (Round 2)
# ========================================
# Issues fixed in this package:
#
# 1. pqcrypto-kyber has AVX2 assembly that MSVC can't compile → PQC removed, stubbed
# 2. Duplicate `hex = "0.4"` in Cargo.toml → fixed
# 3. Missing `randomized` field in HopState struct (hop_plan.rs) → fixed
# 4. Broken escape sequence in status_server.rs → fixed
# 5. Missing TorControl import in status_server.rs → fixed
# 6. toml::from_slice doesn't exist in toml 0.8 → use from_str (config.rs)
# 7. tokio::TcpStream has no try_clone() → use into_split() (tor_control.rs)
# 8. Missing auth_check function (status_server.rs) → implemented
# 9. Wrong FFI calling convention for Windows service (service.rs) → extern "system"
# 10. Unused imports (main.rs, status.rs) → removed
#
# FIXES BELOW:

# ============================================
# FIX 1: Cargo.toml - Remove PQC + fix duplicate
# ============================================
# Replace your Cargo.toml with Cargo.fixed.toml

# ============================================
# FIX 2: src/pqc.rs - Replace with stub
# ============================================
# Replace src/pqc.rs with src/pqc.disabled.rs

# ============================================
# FIX 3: src/hop_plan.rs - Add randomized field
# ============================================
# In the HopState struct, add: randomized: bool,
# 
# Change:
#   struct HopState {
#       order: Vec<usize>,
#       idx: usize,
#       next_epoch_ms: u64,
#   }
# To:
#   struct HopState {
#       order: Vec<usize>,
#       idx: usize,
#       next_epoch_ms: u64,
#       randomized: bool,
#   }
#
# Also add import at top:
#   use rand::seq::SliceRandom;

# ============================================
# FIX 4: src/status_server.rs - Add import + fix escape
# ============================================
# Add at top with other imports:
#   use crate::tor_control::TorControl;
#
# Fix line 190, change:
#   pass.unwrap_or(\"\")
# To:
#   pass.unwrap_or("")

# ============================================
# QUICK FIX COMMANDS (PowerShell)
# ============================================

# Copy Cargo.toml
# Copy-Item Cargo.fixed.toml Cargo.toml -Force

# Copy pqc.rs  
# Copy-Item src\pqc.disabled.rs src\pqc.rs -Force

# Copy hop_plan.rs
# Copy-Item src\hop_plan.fixed.rs src\hop_plan.rs -Force

# Then manually edit status_server.rs:
# 1. Add: use crate::tor_control::TorControl;
# 2. Fix line 190: pass.unwrap_or(\"\") -> pass.unwrap_or("")

# Then rebuild:
# cargo build --release
