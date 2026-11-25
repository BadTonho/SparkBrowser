# 🪟 Window Background & Surface Customization

> 🌍 **Language:** **English** | [Português (pt-BR)](../../pt-BR/features/window-appearance.md)

Spark Browser provides extensive control over the visual surface and transparency of floating windows, dialogs, popovers, and drawers (such as **Browser Settings**, **Quick Settings**, **Customize New Tab**, **Extensions Manager**, and **History/Downloads Drawers**).

---

## 1. Visual Styles

Users can choose between three surface styles in **Settings → Appearance & Layout**:

### ⬛ Matte
- Provides a completely opaque, uniform matte dark or light surface (`#18181b` in dark mode; `#ffffff` in light mode).
- Disables backdrop blur for users who prefer flat, high-contrast surfaces without transparency.

### 💧 Glass
- Floating windows and menus use a translucent black base in dark mode and a translucent white base in light mode. A thin, uniform border defines the edge.
- Settings in a tab uses a flat black or white background depending on the theme. The Spark palette still colors controls and selections.
- Floating surfaces use a gentler blur than Ice.
- **Adjustable:** Follows the opacity and blur sliders below.
- **Glass type:** With Glass selected, a second option picks **Classic** (the look above) or **Liquid glass**: a clearer, lightly tinted surface with a strong blur, boosted color, a bright edge and inner highlights that catch the light. Both follow the opacity and blur sliders.

### ❄️ Ice
A strong blur, close to the frosted look of the COSMIC desktop:
- **Translucent Tint:** Uses a semi-transparent base surface (`rgba(22, 22, 28, var(--modal-surface-opacity))` in dark mode; `rgba(255, 255, 255, var(--modal-surface-opacity))` in light mode).
- **Deep Blur:** Applies hardware-accelerated `backdrop-filter: blur(var(--modal-surface-blur)) saturate(160%)` that diffuses underlying web pages, wallpapers, or desktop windows.
- **Soft Elevation:** A subtle border and deep ambient drop shadows keep the surface legible.

The saved value of Ice is still `frosted`, so a choice made in an earlier version is kept.

---

## 2. Controls & Configuration

Quick Settings uses more pronounced outlines and higher-contrast labels in both themes. Selected options use the current palette’s accent color.

All controls are accessible under **Appearance & Layout** in Browser Settings:

1. **Window Background Style (`settings.window_surface_style`)**:
   - Switch between **Matte**, **Glass** and **Ice** (Matte is enabled by default).
   - **Glass type (`settings.glass_variant`)**: shown only with Glass selected; choose **Classic** (default) or **Liquid glass**.
2. **Window Background Opacity (`settings.window_surface_opacity`)**:
   - A slider from **30%** to **100%** that controls the intensity of the tint. Ice is opaque at 100%, and Glass retains its translucent finish.
   - Default value: **70%**.
3. **Window Blur (`settings.window_surface_blur`)**:
   - Adjusts the diffusion radius from **0 px** to **32 px** (Glass and Ice only).
   - Default value: **20 px**.
4. **Dialog Backdrop Blur (`settings.background_blur_c21c`)**:
   - Controls the separate full-viewport background dimming blur applied behind open dialog overlays (0 to 24 px).

---

## 3. Persistence & Platform Compatibility

- **Persistence:** Values are stored in `safeStorage` under `spark_window_surface_style`, `spark_glass_variant`, `spark_window_surface_opacity`, and `spark_window_surface_blur_px`, persisting across browser sessions.
- **Cross-Platform:** Glass uses CSS for the translucent surface and blur. How much of the background a WebView can blur depends on the rendering engine and which surfaces it composites together.

