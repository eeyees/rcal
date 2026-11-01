# VHS Tape for rcal Demo & Screenshots
Output "assets/demo.gif"

# Terminal Settings
Set Shell "zsh"
Set FontSize 14
Set Width 1440
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
Type "j"
Sleep 600ms
Type "l"
Sleep 600ms
Type "l"
Sleep 600ms
Type "h"
Sleep 600ms

# 2. Switch to Week view (Key: 2)
Type "2"
Sleep 1.5s
Screenshot "assets/week_view.png"
Sleep 1s

# 3. Switch to Agenda view (Key: 4)
Type "4"
Sleep 1.5s
Screenshot "assets/agenda_view.png"
Sleep 1s

# 4. Open Settings Modal (Key: s)
Type "s"
Sleep 1.2s

# Cycle Themes (Key: c)
Type "c"
Sleep 1s
Type "c"
Sleep 1s
Screenshot "assets/theme_modal.png"
Sleep 1s

# Close Modal (Escape) and return to Month view (Key: 1)
Escape
Sleep 1s
Type "1"
Sleep 1.5s

# Quit rcal (Key: q)
Type "q"
Sleep 500ms
