Output assets/demo.gif

# Visual & Window Dimensions
Set Width 1100
Set Height 600
Set Margin 20
Set WindowBar Rings

# Typography (Perfect Box-Drawing & Sparkline alignment)
Set FontFamily "JetBrainsMono Nerd Font, JetBrains Mono, Menlo, monospace"
Set FontSize 15
Set LineHeight 1.15
Set LetterSpacing 0

# Color Theme
Set Theme "Catppuccin Mocha"
Set Padding 18
Set Framerate 30
Set PlaybackSpeed 1.0

# Pre-execution: Hide prompt delay and launch pre-compiled binary instantly
Hide
Type "clear" Enter
Show

Type "./target/release/netpulse"
Sleep 400ms
Enter

# Let live network metrics pulse for 8 seconds
Sleep 15s

# Graceful exit
Type "q"
Sleep 500ms
