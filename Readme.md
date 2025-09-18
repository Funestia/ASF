This is an Armor set finder for monster hunter. Currently the only Supported Game is 4u.
# How to use
* Set your language before you do anything else
* The Required skills column funcions as a sort of bookmark. If you remove the checkmark, the skill isn't required for the Armor but still stays in the column
* The displayed decoration count for sets with torso up isn't accurate. Decorations that should be put in the chest piece will appear twice in the resulting armor set. In theory some results with torso up will be impossible in practice This doesn't happen often though.
* Other tools might find more results than this one. This is by design because all pieces that are strictly worse than another piece are excluded by default for performance reasons.
* If you don't get any results you can try increasing the considered parts per slot value
* If you experience performance issues or run out of ram, try decreasing the considered parts per slot value
* Press the wastebucket button next to an armor piece to exclude Results containing that piece

# Bugs
* Negative skill points on decorations are not accounted for correctly for some skills only.
* If negative skill points on decorations create a loop, the resulting sets won't be valid.
* Changing your language invalidates all charms and relics

These bugs didn't bother me so I didn't fix them, I will do so if they prove to be real issues.

# Missing features
* Currently it is not possible to disable torso up armor pieces
* There is no option to enable all possible relic armor pieces
