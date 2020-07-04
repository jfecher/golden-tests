# Make sure differences of a single space/tab can be spotted.
# Unfortunately, many terminals (including mine) don't support
# coloring the foreground or background of tab characters. So
# instead of trying to highlight the character, goldentests will
# issue a warning for Add lines that contain a tab character.

# Tab used here
print("Hello,	Tab!");

# But space expected here (uncomment line 11 and comment line 14 for a test error):
# # expected stdout:
# Hello, Tab!

# expected stdout:
# Hello,	Tab!
