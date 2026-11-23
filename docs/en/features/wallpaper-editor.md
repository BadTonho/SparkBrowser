# Custom background editor

[Português](../../pt-BR/features/wallpaper-editor.md)

**Customize new tab → Background → Custom → Customize** opens the editor in a tab of its own (`spark://wallpaper`). It is a browser page of its own: it cannot call any browser command, and everything it draws and reads stays on the page.

## Layout

- **Left bar:** two tabs. **Options** lists the looks: **Nebula**, **Space** and **Galaxy**. **Presets** is where you keep your settings: **Save preset** stores a copy of everything you have set under a name, clicking a preset shows it again, and ✕ deletes it.
- **Right bar:** the settings of the chosen look.
- **Center:** the live background, drawn with WebGL and updated as you change a setting.

## The looks

Every look has its own **Shape** and **Motion** settings, and the right bar changes when you choose another look. The values of each look are kept separately: going back to a look finds its sliders as you left them. Colors, Gradient map, Grade and Effects (below) are shared by all of them.

- **Nebula:** clouds that drift, with a field of twinkling stars. Shape: Scale, Billow, Contrast, Stars. Motion: Speed, Direction (forward or backward) and Angle (the way the clouds drift).
- **Space:** stars in depth, coming at you at different speeds over a soft glow. Shape: Density, Size, Twinkle, Depth (how much the layers differ in speed), Glow. Motion: Speed, Direction (toward you or away from you) and Sway (the whole field drifts side to side).
- **Galaxy:** a spiral turning around a bright core, with dust lanes and stars. Shape: Arms, Twist, Core, Dust, Stars, Tilt (how much the disc is seen from the side). Motion: Rotation speed, Direction (clockwise or counterclockwise) and Breathing (the spiral slowly tightens and loosens).

## Settings shared by all looks

- **Colors:** eight swatches (click one to pick a color), **Randomize**, **From image** (reads a picture from your disk and keeps its eight main colors; the picture is not uploaded) and **Active colors**, how many of the eight the gradient uses.
- **Gradient map:** **Blend** (sRGB, Linear, OKLab or OKLCH), **Stops** (drag a handle, or focus it and use the arrow keys, Shift for bigger steps), **Bias** and **Sharpness**. **Reset** restores this section.
- **Grade:** Brightness, Contrast and Saturation.
- **Effects:** **Grain** (Classic or Film, amount, size, color, mid-tone focus, seed, and animation), **Tone** (None or ACES, exposure in EV), **Vignette** and **Dither art** (ordered dither). **Reset** restores this section.

## Using a preset as the new tab background

A preset remembers which look it was made with. Every preset you save appears as a card in **Customize new tab → Background → Custom**, with a small preview of its colors. Click a card to use it as the background of the new tab page (it is drawn live, behind the clock and widgets); click the selected card again to go back to the picture or video background. While a custom background is chosen it replaces the picture or video one. Open new tab pages update on their own when you choose a preset, or when you save a new one in the editor. A copy of the settings is kept with your choice, so deleting the preset in the editor does not change the background.

## Limits for now

Presets and the chosen background stay on this device, in the pages' own storage, and are not part of backups. What you are editing is not saved on its own; the top bar (save, export, presets) and the other looks come later. The drawing pauses while the tab is hidden and is limited to a pixel ratio of 1.5 to keep it light. If the web view has no WebGL, the page says so instead of drawing.
