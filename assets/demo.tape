# VHS Tape for rcal Demo & Screenshots
Output "assets/demo.gif"

# Terminal Settings
Set Shell "zsh"
Set FontSize 14
Set Width 1500
Set Height 820
Set Padding 20
Set WindowBar Colorful
Set Framerate 30

# Launch rcal in clean demo environment
Type "HOME=/tmp/rcal_demo_home ./target/release/rcal"
Enter
Sleep 2s

# 1. Main Hero: Month View Screenshot
Screenshot "assets/screenshot.png"
Sleep 1s

# Navigate in Month view
Type "l"
Sleep 500ms
Type "l"
Sleep 500ms
Type "j"
Sleep 600ms

# Jump back to Today (t) and switch to Week view (2)
Type "t"
Sleep 600ms
Type "2"
Sleep 1.5s

# 2. Week View Screenshot
Screenshot "assets/week_view.png"
Sleep 1.5s

# 3. Switch to Agenda view (Key: 4)
Type "4"
Sleep 1.5s
Screenshot "assets/agenda_view.png"
Sleep 1.5s

# Return to Month view (Key: 1)
Type "1"
Sleep 1.2s

# 4. Open Settings & Themes Modal (Key: s)
Type "s"
Sleep 1.2s

# Cycle Themes (Key: c) to Dracula
Type "c"
Sleep 800ms
Type "c"
Sleep 800ms
Screenshot "assets/theme_modal.png"
Sleep 1.2s

# Close Modal (Escape) and display updated Dracula theme
Escape
Sleep 2s

# Quit rcal (Key: q)
Type "q"
Sleep 500ms
