# Rewwise 🔊

A set of tools for working with Elden Ring and Armored Core 6 soundbanks allowing you to unpack, repack and visualize soundbanks.

## How do I use this
After obtaining a release from the releases tab you can take a .bnk file from either of the games and drag them onto the bnk2json. If all went well it will have created a new folder named after the soundbank containing all the wems as well as a soundbank.json. Once you are done making your edits to the folder's contents you can drag the entire folder back onto the bnk2json which will repack the WEMs and use the soundbank.json to create a new soundbank. This newly created soundbank will be stored with the extension `.created.bnk`. This created soundbank can be loaded instead of the original soundbank using modengine2.

#### Soundbank.json
This file contains describes the event routing, bussing structure, looping of music, and a plethora of other things for this soundbank.

#### WEMs
WEMs contain the actual audio. If you're looking to extract audio this is what you're looking for. You can use [vgmstream](https://vgmstream.org/) to convert from WEM to other, more common, formats.
If you want to put custom audio into a soundbank you will need to convert your audio to a WEM first. Unfortunately converting to a WEM is a bit more complicated and as of now requires Wwise studio itself, [this video illustrates how you can use Wwise studio to convert to WEM](https://www.youtube.com/watch?v=39Oeb4GvxEc).

### Still questions on proper usage?
Check out [this guide](https://docs.google.com/document/d/1lNov-a0DwnMY2yZywH3hFYzuoDfndofguvZmAnLDo-U/edit#heading=h.7dtqo3tlss5x) by Themyys.

## Credits
- Wwiser project for their parsing code
- [Deku](https://github.com/sharksforarms/deku) for the parser code (finally found something worthwhile after doing 3 rewrites using `io::Read` and `io::Write`)
- Shion and SekiroDubi for testing
