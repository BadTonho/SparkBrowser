(() => {
        const root = document.getElementById("overlay");
        const menu = document.getElementById("menu");
        let activeToken = "";
        let isOpen = false;
        let requestSent = false;

        function sendRequest(action) {
          if (!isOpen || requestSent) return;
          requestSent = true;
          const url = new URL(`spark-internal://bookmark-context-menu/${action}`);
          url.searchParams.set("token", activeToken);
          window.location.href = url.toString();
        }

        function dismissOutside(event) {
          if (isOpen && !menu.contains(event.target)) sendRequest("dismiss");
        }

        function dismissEscape(event) {
          if (!isOpen || event.key !== "Escape") return;
          event.preventDefault();
          event.stopImmediatePropagation();
          sendRequest("dismiss");
        }

        window.__sparkSetBookmarkContextMenu = (payload) => {
          if (payload.language) {
            document.documentElement.lang = payload.language.locale === "pt-BR" ? "pt-BR" : "en";
            menu.setAttribute("aria-label", payload.language.label);
            document.title = payload.language.label;
          }
          if (!payload.visible) {
            isOpen = false;
            requestSent = false;
            root.style.display = "none";
            root.setAttribute("aria-hidden", "true");
            return;
          }

          activeToken = payload.token || "";
          requestSent = false;
          isOpen = true;
          root.style.display = "block";
          root.setAttribute("aria-hidden", "false");
          menu.replaceChildren();
          menu.style.backdropFilter = `blur(${Math.min(24, Math.max(0, Number(payload.backdropBlurPx) || 0))}px)`;

          for (const action of payload.actions || []) {
            if (action.heading) {
              const heading = document.createElement("div");
              heading.className = "heading";
              if (/^#[0-9a-f]{6}$/i.test(action.color || "")) {
                const dot = document.createElement("span");
                dot.className = "dot";
                dot.style.background = action.color;
                heading.appendChild(dot);
                menu.style.borderColor = action.color + "66";
              } else {
                menu.style.borderColor = "";
              }
              const name = document.createElement("span");
              name.textContent = action.label;
              heading.appendChild(name);
              menu.appendChild(heading);
              continue;
            }
            const button = document.createElement("button");
            button.type = "button";
            button.setAttribute("role", "menuitem");
            button.dataset.danger = String(Boolean(action.danger));
            const rich = action.icon !== undefined || action.subtitle || action.active;
            if (rich) {
              button.classList.add("rich");
              if (action.active) button.classList.add("active");
              if (/^#[0-9a-f]{6}$/i.test(action.color || "")) button.style.setProperty("--item-color", action.color);
              const icon = document.createElement("span");
              icon.className = "icon";
              if (typeof action.icon === "string" && /^data:image\//.test(action.icon)) {
                const image = document.createElement("img");
                image.alt = "";
                image.draggable = false;
                image.src = action.icon;
                icon.appendChild(image);
              } else {
                icon.textContent = "\u2726";
              }
              const text = document.createElement("span");
              text.className = "text";
              const title = document.createElement("span");
              title.className = "title";
              title.textContent = action.label;
              text.appendChild(title);
              if (action.subtitle) {
                const subtitle = document.createElement("span");
                subtitle.className = "subtitle";
                subtitle.textContent = action.subtitle;
                text.appendChild(subtitle);
              }
              button.append(icon, text);
              if (action.active) button.setAttribute("aria-current", "true");
            } else {
              button.textContent = action.label;
            }
            button.addEventListener("click", (event) => {
              event.preventDefault();
              event.stopPropagation();
              sendRequest(action.id);
            });
            menu.appendChild(button);
          }

          const left = Number(payload.x) || 8;
          const top = Number(payload.y) || 8;
          menu.style.left = `${Math.max(8, Math.min(left, window.innerWidth - menu.offsetWidth - 8))}px`;
          menu.style.top = `${Math.max(8, Math.min(top, window.innerHeight - menu.offsetHeight - 8))}px`;
          menu.querySelector("button")?.focus({ preventScroll: true });
        };

        document.addEventListener("pointerdown", dismissOutside, true);
        window.addEventListener("keydown", dismissEscape, true);
      })();
