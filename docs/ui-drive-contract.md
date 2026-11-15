# MixViu UI drive contract

The `<app>.ui.*` Bus surface belongs to MixViu. Toolkit adapters implement
this vocabulary; changing a toolkit must not change these names or reply fields.

## Keys and text

`ui.key` takes `key` (a case-sensitive string from the complete JSON list
below) and optional `modifiers` (an array of `ctrl`, `command`, `shift`, `alt`).
Duplicates are harmless; order does not matter. `ctrl` and `command` are
synonyms: both set control and the platform command flag. No other spelling,
case folding or whitespace trimming is performed. Unknown keys produce an
`ARGUMENT` error naming the rejected key. A key is pressed and released in
successive passes, with modifiers held around it. A key event does not itself
insert printable text; use `ui.type` for text entry.

Aliases in the list are compatibility names, including literal space, arrow
symbols, browser key codes and numpad spellings. Numpad aliases do not identify
a separate physical key. Digits use `0` or `Digit0`, not `Num0`.

Shift is explicit: for a terminal control chord use, for example,
`{"key":"2","modifiers":["ctrl","shift"]}` for Ctrl+@,
`6` with those modifiers for Ctrl+^, and `Minus` for Ctrl+_. Characters
without a key name (`@`, `^`, `_`, non-Latin text, for example) travel as text
via `ui.type`, not as newly invented key names. Physical terminal input can
carry a base key with Shift as well as the layout's resulting text. Named
punctuation such as `?` remains accepted without inferring a Shift modifier.

The test compares this list with the toolkit table in both directions.

<!-- key-names -->
```json
["⏷","ArrowDown","Down","⏴","ArrowLeft","Left","⏵","ArrowRight","Right","⏶","ArrowUp","Up","Escape","Esc","Tab","Backspace","Enter","Return","NumpadEnter","Insert","Delete","Home","End","PageUp","PageDown","Copy","Cut","Paste"," ","Space",":","Colon",",","Comma","NumpadComma","-","−","Minus","NumpadSubtract",".","Period","NumpadDecimal","+","Plus","NumpadAdd","=","Equal","Equals","NumpadEqual",";","Semicolon","\\","Backslash","/","Slash","NumpadDivide","|","Pipe","?","Questionmark","!","Exclamationmark","[","OpenBracket","BracketLeft","]","CloseBracket","BracketRight","{","OpenCurlyBracket","}","CloseCurlyBracket","`","Backtick","Backquote","Grave","'","Quote","0","Digit0","Numpad0","1","Digit1","Numpad1","2","Digit2","Numpad2","3","Digit3","Numpad3","4","Digit4","Numpad4","5","Digit5","Numpad5","6","Digit6","Numpad6","7","Digit7","Numpad7","8","Digit8","Numpad8","9","Digit9","Numpad9","a","A","KeyA","b","B","KeyB","c","C","KeyC","d","D","KeyD","e","E","KeyE","f","F","KeyF","g","G","KeyG","h","H","KeyH","i","I","KeyI","j","J","KeyJ","k","K","KeyK","l","L","KeyL","m","M","KeyM","n","N","KeyN","o","O","KeyO","p","P","KeyP","q","Q","KeyQ","r","R","KeyR","s","S","KeyS","t","T","KeyT","u","U","KeyU","v","V","KeyV","w","W","KeyW","x","X","KeyX","y","Y","KeyY","z","Z","KeyZ","F1","F2","F3","F4","F5","F6","F7","F8","F9","F10","F11","F12","F13","F14","F15","F16","F17","F18","F19","F20","F21","F22","F23","F24","F25","F26","F27","F28","F29","F30","F31","F32","F33","F34","F35","BrowserBack","BrowserForward","BrowserRefresh","BrowserSearch","BrowserHome","BrowserFavorites","BrowserStop","MediaPlayPause","MediaTrackNext","MediaTrackPrevious","MediaStop","AudioVolumeMute","AudioVolumeDown","AudioVolumeUp","LaunchMail","LaunchApp1","LaunchApp2","ShiftLeft","ShiftRight","ControlLeft","ControlRight","AltLeft","AltRight","SuperLeft","MetaLeft","OSLeft","SuperRight","MetaRight","OSRight","IntlBackslash","CapsLock","NumLock","ScrollLock","PrintScreen","Pause","ContextMenu","Menu","Apps","Fn","FnLock","Globe","Eject","Help","Power","Sleep","Standby","Clear"]
```

## Tree

`ui.tree` returns an object with exactly `root`, `focus` and `nodes`.
`root` and `focus` are decimal unsigned 64-bit id strings or null. `nodes`
is a flat array in depth-first preorder, following each node's `children`
order. Missing child records are skipped. The root is the window's tree root,
or the first supplied node when there is no explicit root. Before any frame,
the command returns `BUSY`.

Each node has the following fields (nullable fields are present even when null):

| Field | Type and meaning |
| --- | --- |
| `id` | Decimal unsigned 64-bit string, unique within this window tree |
| `role` | Case-sensitive MixViu role string from the vocabulary below |
| `label` | String or null; a plain Label's value supplies its label when needed |
| `value` | String or null |
| `placeholder` | String or null |
| `rect` | `[x0,y0,x1,y1]` numbers or null, window-local logical points, x right/y down |
| `enabled` | Boolean, false for a disabled node |
| `focused` | Boolean, whether `id` equals the tree's focus id |
| `selected` | Boolean or null (not applicable) |
| `checked` | `"true"`, `"false"`, `"mixed"` or null (not applicable) |
| `children` | Ordered array of child id strings |
| `depth` | Non-negative integer, root zero |

Logical points are converted to pixels with `window.state`'s pixels per point.
Bounds describe layout geometry; they do not promise visibility or clipping.
Ids are opaque despite their decimal spelling. They remain usable while the
widget identity survives in the current window, with no guarantee across
rebuilds, reordering, removal/recreation, windows or process runs. Re-read the
tree after structural changes. Do not persist ids as application identifiers.

Optional `label` filters by case-insensitive substring; `exact:true` instead
compares the entire label case-sensitively. Optional `role` matches exactly.
Filters retain original ordering, depth, child references, root and focus;
they do not reparent nodes. The focused-node record in input replies uses the
same fields except `depth` is absent.

## Roles

The complete supported role vocabulary follows. It retains existing strings,
including roles not emitted by every widget. The adapter maps each explicitly;
new backend roles require a deliberate contract update. A headless fixture
window also pins the roles actually emitted by representative controls.

<!-- role-names -->
```json
["Unknown","TextRun","Cell","Label","Image","Link","Row","ListItem","ListMarker","TreeItem","ListBoxOption","MenuItem","MenuListOption","Paragraph","GenericContainer","CheckBox","RadioButton","TextInput","Button","DefaultButton","Pane","RowHeader","ColumnHeader","RowGroup","List","Table","LayoutTableCell","LayoutTableRow","LayoutTable","Switch","Menu","MultilineTextInput","SearchInput","DateInput","DateTimeInput","WeekInput","MonthInput","TimeInput","EmailInput","NumberInput","PasswordInput","PhoneNumberInput","UrlInput","Abbr","Alert","AlertDialog","Application","Article","Audio","Banner","Blockquote","Canvas","Caption","Caret","Code","ColorWell","ComboBox","EditableComboBox","Complementary","Comment","ContentDeletion","ContentInsertion","ContentInfo","Definition","DescriptionList","Details","Dialog","DisclosureTriangle","Document","EmbeddedObject","Emphasis","Feed","FigureCaption","Figure","Footer","Form","Grid","GridCell","Group","Header","Heading","Iframe","IframePresentational","ImeCandidate","Keyboard","Legend","LineBreak","ListBox","Log","Main","Mark","Marquee","Math","MenuBar","MenuItemCheckBox","MenuItemRadio","MenuListPopup","Meter","Navigation","Note","PluginObject","ProgressIndicator","RadioGroup","Region","RootWebArea","Ruby","RubyAnnotation","ScrollBar","ScrollView","Search","Section","SectionFooter","SectionHeader","Slider","SpinButton","Splitter","Status","Strong","Suggestion","SvgRoot","Tab","TabList","TabPanel","Term","Time","Timer","TitleBar","Toolbar","Tooltip","Tree","TreeGrid","Video","WebView","Window","PdfActionableHighlight","PdfRoot","GraphicsDocument","GraphicsObject","GraphicsSymbol","DocAbstract","DocAcknowledgements","DocAfterword","DocAppendix","DocBackLink","DocBiblioEntry","DocBibliography","DocBiblioRef","DocChapter","DocColophon","DocConclusion","DocCover","DocCredit","DocCredits","DocDedication","DocEndnote","DocEndnotes","DocEpigraph","DocEpilogue","DocErrata","DocExample","DocFootnote","DocForeword","DocGlossary","DocGlossRef","DocIndex","DocIntroduction","DocNoteRef","DocNotice","DocPageBreak","DocPageFooter","DocPageHeader","DocPageList","DocPart","DocPreface","DocPrologue","DocPullquote","DocQna","DocSubtitle","DocTip","DocToc","ListGrid","Terminal"]
```
