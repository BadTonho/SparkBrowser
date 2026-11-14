(() => {
        const root = document.getElementById("overlay");
        const list = document.getElementById("workspace-list");
        let activeToken = "";
        let isOpen = false;
        let requestSent = false;
        let transitionSequence = 0;
        let listAnimation;

        const OPEN_DURATION_MS = 180;
        const CLOSE_DURATION_MS = 140;
        const EASING = "cubic-bezier(0.16, 1, 0.3, 1)";

        function animateList(from, to, duration) {
          listAnimation?.cancel();
          listAnimation = undefined;

          if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
            list.style.opacity = String(to.opacity);
            list.style.transform = to.transform;
            return;
          }

          listAnimation = list.animate([from, to], {
            duration,
            easing: EASING,
            fill: "both",
          });
        }

        function sendRequest(action, workspaceId) {
          if (!isOpen || requestSent) return;
          requestSent = true;
          const url = new URL(`spark-internal://workspace-switcher/${action}`);
          url.searchParams.set("token", activeToken);
          if (workspaceId) url.searchParams.set("workspace_id", workspaceId);
          window.location.href = url.toString();
        }

        function handlePointerDown(event) {
          if (!list.contains(event.target)) sendRequest("dismiss");
        }

        function handleKeyDown(event) {
          if (event.key !== "Escape") return;
          event.preventDefault();
          event.stopImmediatePropagation();
          sendRequest("dismiss");
        }

        window.__sparkSetWorkspaceSwitcher = (payload) => {
          if (payload.language) {
            document.documentElement.lang = payload.language.locale === "pt-BR" ? "pt-BR" : "en";
            list.setAttribute("aria-label", payload.language.label);
            document.title = payload.language.label;
          }
          const shouldOpen = Boolean(payload.visible);
          const wasVisible = isOpen && root.style.display !== "none";
          const transition = ++transitionSequence;
          requestSent = false;
          activeToken = payload.token || "";
          const requestedBlur = Number(payload.backdropBlurPx);
          const blurPx = Number.isFinite(requestedBlur)
            ? Math.min(24, Math.max(0, requestedBlur))
            : 16;
          list.style.backdropFilter = `blur(${blurPx}px)`;

          if (!shouldOpen) {
            if (!isOpen && root.style.display === "none") return;

            isOpen = false;
            root.setAttribute("aria-hidden", "true");
            list.style.pointerEvents = "none";
            animateList(
              { opacity: 1, transform: "translateY(0)" },
              { opacity: 0, transform: "translateY(-10px)" },
              CLOSE_DURATION_MS
            );
            window.setTimeout(() => {
              if (transition === transitionSequence && !isOpen) {
                root.style.display = "none";
                list.style.pointerEvents = "auto";
              }
            }, CLOSE_DURATION_MS);
            return;
          }

          isOpen = true;
          root.setAttribute("aria-hidden", "false");
          root.style.display = "flex";
          list.style.pointerEvents = "auto";

          if (wasVisible) {
            for (const button of list.querySelectorAll("button[data-workspace-id]")) {
              button.setAttribute(
                "aria-current",
                String(button.dataset.workspaceId === payload.activeWorkspaceId)
              );
            }
            return;
          }

          list.replaceChildren();

          for (const workspace of payload.workspaces || []) {
            const button = document.createElement("button");
            button.type = "button";
            button.dataset.workspaceId = workspace.id;
            button.setAttribute(
              "aria-current",
              String(workspace.id === payload.activeWorkspaceId)
            );

            const icon = document.createElement("span");
            icon.className = "icon";
            icon.setAttribute("aria-hidden", "true");
            icon.textContent = workspace.icon;

            const name = document.createElement("span");
            name.className = "name";
            name.textContent = workspace.name;

            button.append(icon, name);
            button.addEventListener("pointerdown", (event) => {
              event.preventDefault();
              event.stopPropagation();
            });
            button.addEventListener("click", (event) => {
              event.preventDefault();
              event.stopPropagation();
              sendRequest("select", workspace.id);
            });
            list.appendChild(button);
          }

          animateList(
            { opacity: 0, transform: "translateY(-10px)" },
            { opacity: 1, transform: "translateY(0)" },
            OPEN_DURATION_MS
          );
        };

        document.addEventListener("pointerdown", handlePointerDown, true);
        window.addEventListener("keydown", handleKeyDown, true);
      })();
