import {
  FocusTracker,
  isBadgeEnabledForHost,
  isEditableTarget,
  isTriggerShortcut,
} from "./focus";
import { Palette } from "./palette";

class ExtensionController {
  private focusTracker = new FocusTracker();
  private palette = new Palette();
  private badgeElement: HTMLDivElement | null = null;
  private hideBadgeTimer: number | null = null;

  public init(): void {
    this.createBadge();

    // Synchronize palette lifecycle with floating badge visibility:
    // Opening the palette reveals the badge across all sites.
    this.palette.setOnOpen((target) => {
      this.showBadge(target);
    });

    // Closing the palette hides the badge on unlisted sites (non-social platforms),
    // keeping the badge persistently visible only on whitelisted social platforms.
    this.palette.setOnClose(() => {
      if (!isBadgeEnabledForHost(window.location.hostname)) {
        this.hideBadge();
      }
    });

    this.bindEvents();
    this.palette.init().catch((err) => {
      console.error("[Taieagi] Failed to pre-initialize WASM palette:", err);
    });
    console.log("[Taieagi] Content script initialized with shortcut Alt+T / Option+T.");
  }

  private createBadge(): void {
    if (this.badgeElement) {
      return;
    }

    const badge = document.createElement("div");
    badge.id = "taieagi-badge";
    badge.className = "taieagi-badge";
    badge.textContent = "台閩";
    badge.title = "喚起臺灣本土語言快彈詞盤 (Alt+T / Option+T)";
    badge.style.display = "none";

    badge.addEventListener("mousedown", (e) => {
      // Prevent blurring the active input when clicking the badge
      e.preventDefault();
      e.stopPropagation();
    });

    badge.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      const current = (this.focusTracker.getCurrent() ?? document.activeElement) as HTMLElement | null;
      if (current && isEditableTarget(current)) {
        this.togglePalette(current);
      }
    });

    document.body.appendChild(badge);
    this.badgeElement = badge;
  }

  private bindEvents(): void {
    // Track focus on any editable element across the page
    document.addEventListener(
      "focusin",
      (e) => {
        const target = e.target;
        if (isEditableTarget(target)) {
          if (this.hideBadgeTimer !== null) {
            window.clearTimeout(this.hideBadgeTimer);
            this.hideBadgeTimer = null;
          }
          this.focusTracker.onFocus(target);
          // Show the floating badge automatically on focus only for whitelisted social platforms
          if (isBadgeEnabledForHost(window.location.hostname)) {
            this.showBadge(target as HTMLElement);
          }
        }
      },
      true
    );

    document.addEventListener(
      "focusout",
      (e) => {
        const target = e.target;
        if (isEditableTarget(target)) {
          this.hideBadgeTimer = window.setTimeout(() => {
            this.focusTracker.onBlur(target);
            this.hideBadge();
          }, 200);
        }
      },
      true
    );

    // Global shortcut and key listener in capture phase
    window.addEventListener(
      "keydown",
      (e) => {
        // If palette is currently open, handle palette interaction first
        if (this.palette.isOpen()) {
          if (isTriggerShortcut(e)) {
            e.preventDefault();
            e.stopPropagation();
            this.palette.close();
            return;
          }

          const handled = this.palette.handleKeyDown(e);
          if (handled) {
            return;
          }
        }

        // If palette is closed and shortcut is triggered, open palette
        if (!this.palette.isOpen() && isTriggerShortcut(e)) {
          const current = (this.focusTracker.getCurrent() ?? document.activeElement) as HTMLElement | null;
          if (isEditableTarget(current)) {
            e.preventDefault();
            e.stopPropagation();
            this.togglePalette(current);
          }
        }
      },
      true
    );
  }

  private showBadge(target: HTMLElement): void {
    if (!this.badgeElement) {
      return;
    }

    const rect = target.getBoundingClientRect();
    if (rect.width === 0 && rect.height === 0) {
      return;
    }

    // Align vertically with the target input center, clamped within the viewport
    const badgeHeight = 22;
    const targetCenterY = rect.top + rect.height / 2;
    const clampedTop = Math.min(
      Math.max(10, targetCenterY - badgeHeight / 2),
      window.innerHeight - badgeHeight - 10
    );

    this.badgeElement.style.top = `${clampedTop}px`;
    this.badgeElement.style.display = "flex";
  }

  private hideBadge(): void {
    if (this.badgeElement) {
      this.badgeElement.style.display = "none";
    }
  }

  public togglePalette(target: HTMLElement): void {
    this.palette.toggle(target);
  }
}

// Instantiate and start when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    new ExtensionController().init();
  });
} else {
  new ExtensionController().init();
}
