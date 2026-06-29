{ config, pkgs, lib, ... }:
{
  # INT-024/054/056: REGREET login mode -- cage + ReGreet, candy-neon themed.
  # The migration target (replacing tuigreet on metal once VM-proven).
  services.greetd.enable = true;
  programs.regreet.enable = true;

  time.timeZone = "America/Chicago";  # INT-054: Central, matches host

  # INT-056: greetd 'greeter' user must be in input/seat/video to read
  # /dev/input/event* -- without this ReGreet renders but takes no keyboard.
  users.users.greeter.extraGroups = [ "input" "seat" "video" ];

  # INT-054: candy-neon forest theme (GTK4 CSS). VM-proven at facd128c.
  programs.regreet.extraCss = ''
    /* INT-054 candy-neon GLASS theme -- forest greeter, GTK4.
       Near-black green base, translucent glass card, neon-lime glow, aqua focus. */
    window, .background {
      background-color: #0a0f0c;
      background-image: radial-gradient(circle at 50% 35%, #12200f 0%, #0a0f0c 55%, #060a07 100%);
    }
    box, grid { background-color: transparent; }

    /* The login card: translucent glass with a top-edge light catch + outer glow. */
    .horizontal:not(button), window > box {
      background-color: rgba(16, 22, 14, 0.72);
      border: 1px solid rgba(57, 255, 20, 0.35);
      border-top: 1px solid rgba(120, 255, 90, 0.55);
      border-radius: 16px;
      box-shadow: 0 0 28px rgba(57, 255, 20, 0.18),
                  inset 0 1px 0 rgba(180, 255, 150, 0.18);
    }

    label, .greeter {
      color: #d8f5d0;
      font-family: "JetBrainsMono Nerd Font", monospace;
      text-shadow: 0 0 4px rgba(57, 255, 20, 0.25);
    }

    /* Clock: big glowing lime. */
    .clock {
      color: #6dff3c;
      font-family: "JetBrainsMono Nerd Font", monospace;
      font-size: 2.4em;
      font-weight: bold;
      text-shadow: 0 0 12px rgba(57, 255, 20, 0.55),
                   0 0 24px rgba(57, 255, 20, 0.25);
    }

    /* Entry: glassy inset field, lime border, glow on focus. */
    entry {
      background-color: rgba(8, 12, 8, 0.85);
      color: #6dff3c;
      caret-color: #6dff3c;
      border: 1.5px solid rgba(57, 255, 20, 0.5);
      border-radius: 10px;
      padding: 9px 14px;
      box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.6),
                  inset 0 1px 0 rgba(120, 255, 90, 0.12);
    }
    entry:focus {
      border-color: #50dcff;
      box-shadow: 0 0 14px rgba(80, 220, 255, 0.5),
                  inset 0 2px 6px rgba(0, 0, 0, 0.6);
    }

    /* Buttons: aqua glass, fill-on-hover. */
    button {
      background-image: linear-gradient(rgba(28, 40, 26, 0.9), rgba(14, 22, 12, 0.9));
      color: #50dcff;
      border: 1.5px solid rgba(80, 220, 255, 0.65);
      border-radius: 10px;
      padding: 7px 18px;
      box-shadow: inset 0 1px 0 rgba(150, 230, 255, 0.18),
                  0 0 10px rgba(80, 220, 255, 0.12);
      text-shadow: 0 0 4px rgba(80, 220, 255, 0.4);
    }
    button:hover {
      background-image: linear-gradient(#50dcff, #3bb8e0);
      color: #06120a;
      box-shadow: 0 0 16px rgba(80, 220, 255, 0.55);
    }
    button:active { background-image: linear-gradient(#3bb8e0, #2f9ec0); }

    /* User / Session fields are comboboxes wrapping an entry. */
    combobox,
    combobox > box,
    combobox box.linked,
    combobox button {
      background-color: transparent;
      background-image: none;
      border: none;
      box-shadow: none;
      outline: none;
      color: #d8f5d0;
    }
    combobox entry {
      background-color: rgba(8, 12, 8, 0.85);
      color: #6dff3c;
      border: 1.5px solid rgba(57, 255, 20, 0.5);
      border-radius: 10px;
      box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.6);
    }
    combobox entry:focus {
      border-color: #50dcff;
      box-shadow: 0 0 14px rgba(80, 220, 255, 0.5),
                  inset 0 2px 6px rgba(0, 0, 0, 0.6);
    }
    combobox arrow { color: #ff7b6b; }

    button.image-button {
      background-image: none;
      background-color: rgba(20, 28, 16, 0.6);
      color: #ffb347;
      border: 1px solid rgba(255, 179, 71, 0.5);
      border-radius: 8px;
      box-shadow: none;
      padding: 4px 8px;
      min-width: 0;
    }
    button.image-button:hover {
      background-color: rgba(255, 179, 71, 0.2);
      border-color: rgba(255, 179, 71, 0.8);
    }
    button.image-button image { color: #ffb347; }

    frame, frame > border, .frame {
      border: none;
      box-shadow: none;
      background: none;
      background-color: transparent;
    }
    separator {
      background-color: transparent;
      background-image: none;
      min-height: 0;
      min-width: 0;
      border: none;
    }
  '';

  # INT-054: pre-select mango as the default session so login is just
  # password -> Enter (no typing the session name; "MangoWM" vs "mango"
  # free-text was the bounce-back trap). command matches the .desktop Exec.
  programs.regreet.settings.default_session = {
    command = "mango -c /home/christian/.config/mango/config.conf";
    user = "christian";
  };
}
