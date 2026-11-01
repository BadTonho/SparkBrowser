(() => {
        const root = document.getElementById("overlay");
        const menu = document.getElementById("menu");
        let activeToken = "";
        let isOpen = false;
        let requestSent = false;

        function sendRequest(action) {
          if (!isOpen || requestSent) return;
          requestSent = true;
          const url = new URL(`veloce-internal://bookmark-context-menu/${action}`);
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

        window.__veloceSetBookmarkContextMenu = (payload) => {
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
            const button = document.createElement("button");
            button.type = "button";
            button.setAttribute("role", "menuitem");
            button.dataset.danger = String(Boolean(action.danger));
            button.textContent = action.label;
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
