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
    /* INT-054 candy-neon GLASS theme -- mirrors faelight-logout's pastel-candy
       palette (rose/mint/lavender/lime) + its two-state glow: soft at rest,
       dramatic light-up on focus/hover. Login & logout are visual siblings. */
    window, .background {
      background-color: rgba(8, 13, 8, 0.94);
      background-image: radial-gradient(circle at 50% 35%, #12200f 0%, #0a0f0c 55%, #060a07 100%);
    }
    box, grid { background-color: transparent; }
    .horizontal:not(button), window > box {
      background-color: rgba(12, 20, 4, 0.82);
      border: 2px solid #97C459;
      border-top: 2px solid #C0DD97;
      border-radius: 18px;
      box-shadow: 0 0 22px 2px rgba(151, 196, 89, 0.45),
                  inset 0 0 14px rgba(151, 196, 89, 0.18),
                  inset 0 1px 0 rgba(192, 221, 151, 0.30);
    }
    label, .greeter {
      color: #C0DD97;
      font-family: "JetBrainsMono Nerd Font", monospace;
      text-shadow: 0 0 6px rgba(151, 196, 89, 0.40);
    }
    .clock {
      color: #C0DD97;
      font-family: "JetBrainsMono Nerd Font", monospace;
      font-size: 2.5em;
      font-weight: bold;
      text-shadow: 0 0 14px rgba(151, 196, 89, 0.65),
                   0 0 28px rgba(151, 196, 89, 0.30);
    }
    entry {
      background-color: rgba(12, 20, 4, 0.88);
      color: #C0DD97;
      caret-color: #9FE1CB;
      border: 2px solid #97C459;
      border-radius: 12px;
      padding: 10px 15px;
      box-shadow: 0 0 14px 1px rgba(151, 196, 89, 0.45),
                  inset 0 0 10px rgba(151, 196, 89, 0.18);
    }
    entry:focus {
      border: 3px solid #5DCAA5;
      color: #9FE1CB;
      caret-color: #9FE1CB;
      box-shadow: 0 0 30px 3px rgba(93, 202, 165, 0.90),
                  inset 0 0 14px rgba(93, 202, 165, 0.30);
    }
    button {
      background-color: #06140f;
      color: #5DCAA5;
      border: 2px solid #5DCAA5;
      border-radius: 12px;
      padding: 8px 20px;
      box-shadow: 0 0 14px 1px rgba(93, 202, 165, 0.45),
                  inset 0 0 10px rgba(93, 202, 165, 0.18);
      text-shadow: 0 0 6px rgba(93, 202, 165, 0.45);
    }
    button:hover {
      border: 3px solid #9FE1CB;
      color: #9FE1CB;
      box-shadow: 0 0 30px 3px rgba(93, 202, 165, 0.90),
                  inset 0 0 14px rgba(93, 202, 165, 0.30);
    }
    button:active { background-color: #0a1c14; }
    combobox,
    combobox > box,
    combobox box.linked,
    combobox button {
      background-color: transparent;
      background-image: none;
      border: none;
      box-shadow: none;
      outline: none;
      color: #C0DD97;
    }
    combobox entry {
      background-color: rgba(12, 20, 4, 0.88);
      color: #C0DD97;
      border: 2px solid #97C459;
      border-radius: 12px;
      box-shadow: 0 0 14px 1px rgba(151, 196, 89, 0.45),
                  inset 0 0 10px rgba(151, 196, 89, 0.18);
    }
    combobox entry:focus {
      border: 3px solid #5DCAA5;
      color: #9FE1CB;
      box-shadow: 0 0 30px 3px rgba(93, 202, 165, 0.90),
                  inset 0 0 14px rgba(93, 202, 165, 0.30);
    }
    combobox arrow { color: #ED93B1; }
    button.image-button {
      background-image: none;
      background-color: #1a0a11;
      color: #ED93B1;
      border: 2px solid #ED93B1;
      border-radius: 12px;
      box-shadow: 0 0 14px 1px rgba(237, 147, 177, 0.45),
                  inset 0 0 10px rgba(237, 147, 177, 0.18);
      padding: 6px 10px;
      min-width: 0;
    }
    button.image-button:hover {
      background-color: #1a0a11;
      border: 3px solid #F4C0D1;
      color: #F4C0D1;
      box-shadow: 0 0 30px 3px rgba(237, 147, 177, 0.90),
                  inset 0 0 14px rgba(237, 147, 177, 0.30);
    }
    button.image-button image { color: #ED93B1; }
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
      opacity: 0;
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
