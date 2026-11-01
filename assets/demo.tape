# VHS Tape for rcal Demo
# Output file
Output assets/demo.gif

# Terminal Settings
Set Shell "zsh"
Set FontSize 15
Set Width 1200
Set Height 720
Set Padding 20
Set WindowBar Colorful
Set Framerate 30

# Launch rcal
Type "./target/release/rcal"
Enter
Sleep 2s

# Navigate in Month view
Type "j"
Sleep 600ms
Type "l"
Sleep 600ms
Type "l"
Sleep 600ms

# Switch to Week view (Key: 2 or w)
Type "2"
Sleep 2s

# Switch to Day view (Key: 3 or d)
Type "3"
Sleep 2s

# Switch to Agenda view (Key: 4 or a)
Type "4"
Sleep 2s

# Back to Month view (Key: 1 or m)
Type "1"
Sleep 1.5s

# Open Settings Modal (Key: s)
Type "s"
Sleep 1.5s

# Cycle Themes (Key: c)
Type "c"
Sleep 1s
Type "c"
Sleep 1s
Type "c"
Sleep 1.5s

# Close Modal (Escape)
Escape
Sleep 1.5s

# Quit rcal (Key: q)
Type "q"
Sleep 1s
