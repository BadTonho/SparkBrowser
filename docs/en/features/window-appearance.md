# 🪟 Window Background & Surface Customization

> 🌍 **Language:** **English** | [Português (pt-BR)](../../pt-BR/features/window-appearance.md)

Spark Browser provides extensive control over the visual surface and transparency of floating windows, dialogs, popovers, and drawers (such as **Browser Settings**, **Quick Settings**, **Customize New Tab**, **Extensions Manager**, and **History/Downloads Drawers**).

---

## 1. Visual Styles

Users can choose between two primary surface styles in **Settings → Appearance & Layout**:

### 🪟 Frosted Glass (Jelly Blur)
A modern translucent glassmorphism aesthetic:
- **Translucent Tint:** Uses a semi-transparent base surface (`rgba(22, 22, 28, var(--modal-surface-opacity))` in dark mode; `rgba(255, 255, 255, var(--modal-surface-opacity))` in light mode).
- **Deep Frosted Blur:** Applies hardware-accelerated `backdrop-filter: blur(var(--modal-surface-blur)) saturate(160%)` that dynamically diffuses underlying web pages, wallpapers, or desktop windows.
- **Specular Border & Soft Elevation:** Subtle glowing specular borders and deep ambient drop shadows for elegant contrast and legibility.

### ⬛ Solid Matte
- Provides a completely opaque, uniform matte dark or light surface (`#18181b` in dark mode; `#ffffff` in light mode).
- Disables backdrop blur for users who prefer flat, high-contrast surfaces without transparency.

---

## 2. Controls & Configuration

All controls are accessible under **Appearance & Layout** in Browser Settings:

1. **Window Background Style (`settings.window_surface_style`)**:
   - Switch between **Frosted Glass** and **Solid Matte** (Solid Matte is enabled by default).
2. **Window Background Opacity (`settings.window_surface_opacity`)**:
   - A fine-tuning slider ranging from **30%** (ultra-translucent jelly aesthetic) to **100%** (fully opaque).
   - Default value: **70%**.
3. **Frosted Glass Blur (`settings.window_surface_blur`)**:
   - Adjusts the frosted diffusion radius from **0 px** to **32 px**.
   - Default value: **20 px**.
4. **Dialog Backdrop Blur (`settings.background_blur_c21c`)**:
   - Controls the separate full-viewport background dimming blur applied behind open dialog overlays (0 to 24 px).

---

## 3. Persistence & Platform Compatibility

- **Persistence:** Values are stored in `safeStorage` under `spark_window_surface_style`, `spark_window_surface_opacity`, and `spark_window_surface_blur_px`, persisting across browser sessions.
- **Cross-Platform:** Built using standard CSS custom properties and hardware-accelerated CSS `backdrop-filter`, ensuring smooth, zero-latency performance across both **Windows** (WebView2) and **Linux** (WebKitGTK and modern Wayland/X11 compositors).

