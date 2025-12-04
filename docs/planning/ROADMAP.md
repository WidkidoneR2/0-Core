# 🗺️ Faelight Forest Development Roadmap

**Current Version:** 2.8.0 - Foundational Intelligence ✅  
**Last Updated:** December 03, 2025  
**Roadmap Version:** 5.0 - Architectural Refinement

---

## 📐 Architectural Principles

**Core Philosophy:**
1. **Separation of Concerns** - Theme engine is standalone, FCM consumes it
2. **Right Tool for Job** - Python for algorithms, Fish for UX, Git for versioning
3. **Composability** - Atomic packages are composable building blocks
4. **Declarative Over Imperative** - State what you want, not how to get it
5. **Proven Over Custom** - Leverage Git, Python stdlib, standard tools

**Key Architectural Decisions:**
- ✅ Theme Engine: Standalone Python package with CLI API
- ✅ Dependency Resolution: Python + networkx (not Fish)
- ✅ Snapshots: Git tags + minimal metadata (not custom YAML)
- ✅ Atomic Packages: MANDATORY for v3.0 (non-negotiable)

---

## ✅ Version 2.7.2 - Security Hardening (COMPLETE)

### Goals: Harden System Security

**Achievements:**
- ✅ Lynis Integration (Hardening Index: 68 → 71)
- ✅ Kernel hardening (9 critical settings)
- ✅ Fail2ban operational
- ✅ arch-audit vulnerability scanning
- ✅ Security aliases and weekly routine
- ✅ Complete documentation rewrite

**Impact:** Enterprise-grade security hardening achieved  
**Time Spent:** 2-3 hours

---

## ✅ Version 2.8.0 - Foundational Intelligence (COMPLETE)

### Goals: Build Safety Net Before Major Development

**Achievements:**
- ✅ **dot-doctor**: Comprehensive health monitoring
  - 8/8 Stow packages healthy
  - 3/3 services running
  - 17/17 binaries found
- ✅ **keyscan**: Keybinding conflict detection
  - 101 bindings analyzed
  - 0 conflicts (4 found and resolved!)
- ✅ All conflicts resolved, system optimized
- ✅ Complete documentation (TOOLING.md)

**Impact:** Safety infrastructure for Theme Engine development  
**Time Spent:** 3-4 hours

---

## 🎨 Version 2.8.1 - Theme Engine Foundation (Standalone Package)

### Goals: Build Reusable, Decoupled Theme Engine

**Why Standalone:**
- Reusable outside FCM
- Clean API boundaries
- Easier testing and maintenance
- Potential open-source project

**Workflow:**
````
Design → Structure → Research → Document → Ready for Implementation
````

---

### Phase 1A: Project Structure & Setup (1 hour)

**Goal:** Create properly structured Python package

**Create Separate Repository:**
````
faelight-theme-engine/
├── pyproject.toml           # Python package metadata
├── README.md                # Standalone documentation
├── LICENSE                  # MIT or similar
├── .gitignore
├── faelight_theme/
│   ├── __init__.py
│   ├── extract.py           # Color extraction (stub)
│   ├── palette.py           # Palette generation (stub)
│   ├── contrast.py          # WCAG checking (stub)
│   ├── render.py            # Template rendering (stub)
│   ├── cli.py               # Command-line interface
│   └── templates/           # Empty for now
├── tests/
│   ├── __init__.py
│   └── test_extract.py
└── examples/
    └── example-palette.json
````

**Tasks:**
- [ ] Create `faelight-theme-engine/` directory (OUTSIDE dotfiles)
- [ ] Initialize Git repository
- [ ] Create Python package structure
- [ ] Write `pyproject.toml` with metadata:
````toml
  [project]
  name = "faelight-theme"
  version = "0.1.0"
  description = "Generate beautiful themes from wallpapers"
  dependencies = [
      "click",
      "Pillow",
  ]
````
- [ ] Create comprehensive `README.md`
- [ ] Add MIT license
- [ ] Create stub Python modules with docstrings
- [ ] Make installable: `pip install -e .`
- [ ] Test import: `python -c "import faelight_theme"`

**Deliverable:** Empty but properly structured Python package ✅

---

### Phase 1B: Research & Design (2-3 hours)

**Goal:** Make informed technical decisions before coding

**Study Existing Solutions:**
- [ ] Analyze pywal source code (GitHub)
  - Color extraction approach
  - Palette generation logic
  - Template system
  - What works, what doesn't
- [ ] Research Python color libraries:
  - [ ] Test `colorz` (kmeans clustering)
  - [ ] Test `colorgram.py` (quantization)
  - [ ] Test PIL/Pillow manual extraction
  - [ ] Compare results with sample images
- [ ] Study color theory resources:
  - [ ] WCAG contrast ratios (AA: 4.5:1, AAA: 7:1)
  - [ ] Color harmony (complementary, analogous, triadic)
  - [ ] Accessibility guidelines
  - [ ] Terminal color roles (background, foreground, ANSI 0-15)

**Design Decisions:**
- [ ] **Choose extraction library** (document why)
  - Criteria: accuracy, speed, dependencies, maintenance
- [ ] **Design palette JSON structure:**
````json
  {
    "source": "/path/to/wallpaper.jpg",
    "timestamp": "2025-12-03T20:00:00Z",
    "background": "#0f1c16",
    "foreground": "#e8f5d5",
    "accent": "#7fb069",
    "colors": {
      "color0": "#...",  # black
      "color1": "#...",  # red
      ...
      "color15": "#..." # bright white
    },
    "metadata": {
      "contrast_ratio": 10.5,
      "brightness": "dark"
    }
  }
````
- [ ] **Design CLI interface:**
````bash
  # Extract colors from wallpaper → JSON
  faelight-theme extract wallpaper.jpg > palette.json
  
  # Render template with palette
  faelight-theme render palette.json --target kitty > kitty.conf
  faelight-theme render palette.json --all --output-dir ./generated
  
  # Validate palette
  faelight-theme validate palette.json
  
  # Preview palette (terminal colors)
  faelight-theme preview palette.json
````
- [ ] **Document algorithm approach** (flowchart)

**Documentation:**
- [ ] Create `DESIGN.md` in theme engine repo
- [ ] Document algorithm flowchart (Mermaid diagram)
- [ ] Document color role mapping
- [ ] Create 3+ example palettes (dark, light, vibrant)
- [ ] Document decision rationale

**Deliverable:** Clear design, tool selection documented, ready to code ✅

---

**Estimated Time:** 3-4 hours  
**Dependencies:** v2.8.0 complete ✅  
**Deliverables:**
- ✅ Standalone Python package structure
- ✅ Clear architectural design
- ✅ Tool selection with rationale
- ✅ Ready for implementation

**Next:** v2.8.2 implements color extraction!

---

## 🎨 Version 2.8.2 - Color Extraction Implementation

### Goals: Implement Core Extraction & Palette Generation

**Work Location:** `faelight-theme-engine/` directory (NOT dotfiles!)

**Workflow:**
````
Implement → Test → Validate → Document → Ship
````

---

### Implementation: Color Extraction Module

**File:** `faelight_theme/extract.py`

**Core Function:**
````python
from PIL import Image
import colorgram  # or chosen library

def extract_colors(image_path: str, num_colors: int = 8) -> list[tuple[int, int, int]]:
    """
    Extract dominant colors from wallpaper.
    
    Args:
        image_path: Path to wallpaper image
        num_colors: Number of colors to extract (default: 8)
    
    Returns:
        List of RGB tuples
    """
    # Implementation using chosen library
    pass

def rgb_to_hex(rgb: tuple[int, int, int]) -> str:
    """Convert RGB tuple to hex string."""
    return f"#{rgb[0]:02x}{rgb[1]:02x}{rgb[2]:02x}"

def calculate_brightness(rgb: tuple[int, int, int]) -> float:
    """Calculate perceived brightness (0-255)."""
    # Using luminance formula
    return (0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2])

def sort_by_brightness(colors: list) -> list:
    """Sort colors from dark to light."""
    return sorted(colors, key=lambda c: calculate_brightness(c))
````

**Tasks:**
- [ ] Install chosen library: `pip install colorgram.py` (or selected)
- [ ] Implement `extract_colors()` function
- [ ] Implement `rgb_to_hex()` helper
- [ ] Implement `calculate_brightness()` helper
- [ ] Implement `sort_by_brightness()` helper
- [ ] Filter similar colors (color distance > threshold)
- [ ] Write unit tests:
````python
  def test_extract_colors():
      colors = extract_colors("tests/fixtures/dark-wallpaper.jpg")
      assert len(colors) == 8
      assert all(isinstance(c, tuple) for c in colors)
````

---

### Implementation: Palette Generation Module

**File:** `faelight_theme/palette.py`

**Core Function:**
````python
def generate_palette(base_colors: list[tuple]) -> dict:
    """
    Generate 16-color ANSI palette from extracted colors.
    
    Args:
        base_colors: List of RGB tuples from extraction
    
    Returns:
        Complete palette dictionary
    """
    sorted_colors = sort_by_brightness(base_colors)
    
    # Determine theme brightness
    avg_brightness = sum(calculate_brightness(c) for c in base_colors) / len(base_colors)
    is_dark = avg_brightness < 128
    
    # Select background and foreground
    background = sorted_colors[0] if is_dark else sorted_colors[-1]
    foreground = sorted_colors[-1] if is_dark else sorted_colors[0]
    
    # Generate ANSI colors (map extracted colors to ANSI roles)
    ansi_colors = {
        "color0": sorted_colors[0],   # black
        "color1": find_closest_hue(base_colors, "red"),
        "color2": find_closest_hue(base_colors, "green"),
        # ... etc
    }
    
    return {
        "background": rgb_to_hex(background),
        "foreground": rgb_to_hex(foreground),
        "colors": {k: rgb_to_hex(v) for k, v in ansi_colors.items()}
    }
````

**Tasks:**
- [ ] Implement `generate_palette()` function
- [ ] Implement `find_closest_hue()` helper (color theory!)
- [ ] Generate background/foreground intelligently
- [ ] Generate 8 normal ANSI colors (0-7)
- [ ] Generate 8 bright ANSI colors (8-15)
- [ ] Add JSON export function
- [ ] Write unit tests

---

### Implementation: Contrast Validation

**File:** `faelight_theme/contrast.py`

**Core Functions:**
````python
def relative_luminance(rgb: tuple) -> float:
    """Calculate relative luminance (WCAG formula)."""
    pass

def contrast_ratio(color1: str, color2: str) -> float:
    """Calculate WCAG contrast ratio between two hex colors."""
    # Convert hex → RGB → luminance → ratio
    pass

def validate_palette(palette: dict) -> dict:
    """
    Validate palette meets accessibility standards.
    
    Returns:
        {
            "valid": True/False,
            "warnings": [list of issues],
            "ratios": {
                "bg_fg": 10.5,
                "bg_accent": 4.8
            }
        }
    """
    pass

def auto_adjust_contrast(palette: dict, min_ratio: float = 7.0) -> dict:
    """Auto-adjust colors to meet contrast requirements."""
    pass
````

**Tasks:**
- [ ] Implement WCAG contrast calculation
- [ ] Implement `validate_palette()` function
- [ ] Validate background vs foreground (min 7:1 for AAA)
- [ ] Validate background vs accent colors (min 4.5:1 for AA)
- [ ] Implement `auto_adjust_contrast()` (lighten/darken as needed)
- [ ] Write unit tests

---

### Implementation: CLI Interface

**File:** `faelight_theme/cli.py`
````python
import click
import json
from .extract import extract_colors
from .palette import generate_palette
from .contrast import validate_palette

@click.group()
@click.version_option(version="0.1.0")
def cli():
    """Faelight Theme Engine - Generate themes from wallpapers."""
    pass

@cli.command()
@click.argument('image_path', type=click.Path(exists=True))
@click.option('--num-colors', default=8, help='Number of colors to extract')
@click.option('--output', '-o', help='Output file (default: stdout)')
def extract(image_path, num_colors, output):
    """Extract colors from wallpaper and generate palette JSON."""
    colors = extract_colors(image_path, num_colors)
    palette = generate_palette(colors)
    
    # Add metadata
    palette['source'] = image_path
    palette['timestamp'] = datetime.now().isoformat()
    
    # Validate
    validation = validate_palette(palette)
    if not validation['valid']:
        click.echo(f"⚠️  Warnings: {validation['warnings']}", err=True)
    
    # Output
    result = json.dumps(palette, indent=2)
    if output:
        with open(output, 'w') as f:
            f.write(result)
    else:
        click.echo(result)

@cli.command()
@click.argument('palette_file', type=click.Path(exists=True))
def validate(palette_file):
    """Validate palette contrast ratios."""
    with open(palette_file) as f:
        palette = json.load(f)
    
    result = validate_palette(palette)
    
    if result['valid']:
        click.echo("✅ Palette meets accessibility standards!")
    else:
        click.echo("⚠️  Issues found:")
        for warning in result['warnings']:
            click.echo(f"  - {warning}")
    
    click.echo(f"\nContrast ratios:")
    for key, ratio in result['ratios'].items():
        click.echo(f"  {key}: {ratio:.2f}:1")

if __name__ == '__main__':
    cli()
````

**Tasks:**
- [ ] Install Click: `pip install click`
- [ ] Implement `extract` command
- [ ] Implement `validate` command
- [ ] Add `--help` documentation for all commands
- [ ] Add error handling (file not found, invalid image, etc.)
- [ ] Test CLI: `faelight-theme extract wallpaper.jpg`
- [ ] Test CLI: `faelight-theme validate palette.json`

---

### Testing Phase

**Tasks:**
- [ ] Create `tests/fixtures/` with test wallpapers:
  - [ ] dark-wallpaper.jpg (dark background)
  - [ ] light-wallpaper.jpg (light background)
  - [ ] colorful-wallpaper.jpg (vibrant colors)
  - [ ] muted-wallpaper.jpg (desaturated)
  - [ ] gradient-wallpaper.jpg (smooth gradient)
- [ ] Test extraction with all fixtures
- [ ] Verify contrast ratios are acceptable
- [ ] Document edge cases:
  - Very dark images (all black)
  - Very light images (all white)
  - Monochrome images
  - Low-color images
- [ ] Run pytest: `pytest tests/`
- [ ] Achieve >80% code coverage

---

### Documentation

**Tasks:**
- [ ] Update `README.md` with usage examples
- [ ] Document CLI commands
- [ ] Add example palettes to `examples/`
- [ ] Document limitations and edge cases
- [ ] Add contributing guidelines (if planning to open source)

---

**Estimated Time:** 4-5 hours  
**Dependencies:** v2.8.1 complete ✅  
**Deliverables:**
- ✅ Working color extraction
- ✅ Palette generation algorithm implemented
- ✅ WCAG contrast validation working
- ✅ Functional CLI (`faelight-theme extract`)
- ✅ Unit tests passing (>80% coverage)
- ✅ Documentation complete

**Next:** v2.8.3 adds template rendering!

---

## 🎨 Version 2.8.3 - Template System & Rendering

### Goals: Implement Jinja2 Template Rendering

**Work Location:** `faelight-theme-engine/` directory

**Workflow:**
````
Create Templates → Implement Renderer → Test → Validate → Ship
````

---

### Create Template System

**Template Directory Structure:**
````
faelight-theme-engine/
└── faelight_theme/
    └── templates/
        ├── kitty.conf.j2
        ├── hyprland-colors.conf.j2
        ├── waybar-style.css.j2
        ├── mako.conf.j2
        └── theme.json.j2
````

**Example:** `templates/kitty.conf.j2`
````jinja2
# ═══════════════════════════════════════════════════════════
# 🎨 Faelight Theme - Generated from Wallpaper
# Source: {{ source }}
# Generated: {{ timestamp }}
# ═══════════════════════════════════════════════════════════

# Background and foreground
background {{ background }}
foreground {{ foreground }}

# Black
color0  {{ colors.color0 }}
color8  {{ colors.color8 }}

# Red
color1  {{ colors.color1 }}
color9  {{ colors.color9 }}

# Green
color2  {{ colors.color2 }}
color10 {{ colors.color10 }}

# Yellow
color3  {{ colors.color3 }}
color11 {{ colors.color11 }}

# Blue
color4  {{ colors.color4 }}
color12 {{ colors.color12 }}

# Magenta
color5  {{ colors.color5 }}
color13 {{ colors.color13 }}

# Cyan
color6  {{ colors.color6 }}
color14 {{ colors.color14 }}

# White
color7  {{ colors.color7 }}
color15 {{ colors.color15 }}

# Cursor
cursor {{ foreground }}
cursor_text_color {{ background }}

# Selection
selection_foreground {{ background }}
selection_background {{ colors.color4 }}
````

**Tasks:**
- [ ] Install Jinja2: `pip install jinja2`
- [ ] Create `templates/` directory
- [ ] Create Kitty template (complete with all colors)
- [ ] Create Hyprland colors template
- [ ] Create Waybar CSS template (with variables)
- [ ] Create Mako config template
- [ ] Create theme.json template (for Omarchy custom format)
- [ ] Test templates render without errors

---

### Implement Rendering Module

**File:** `faelight_theme/render.py`
````python
from jinja2 import Environment, FileSystemLoader, TemplateNotFound
from pathlib import Path
import json

class ThemeRenderer:
    """Render configuration templates with palette colors."""
    
    def __init__(self, template_dir: str = None):
        if template_dir is None:
            # Use package templates
            template_dir = Path(__file__).parent / "templates"
        
        self.env = Environment(
            loader=FileSystemLoader(str(template_dir)),
            trim_blocks=True,
            lstrip_blocks=True
        )
    
    def render(self, palette: dict, template_name: str) -> str:
        """
        Render template with palette.
        
        Args:
            palette: Color palette dictionary
            template_name: Template filename (e.g., 'kitty.conf.j2')
        
        Returns:
            Rendered configuration string
        
        Raises:
            TemplateNotFound: If template doesn't exist
        """
        try:
            template = self.env.get_template(template_name)
            return template.render(**palette)
        except TemplateNotFound:
            raise ValueError(f"Template not found: {template_name}")
    
    def render_all(self, palette: dict, output_dir: Path) -> dict[str, Path]:
        """
        Render all templates.
        
        Returns:
            Dictionary mapping template names to output paths
        """
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)
        
        results = {}
        for template_file in self.env.list_templates():
            if not template_file.endswith('.j2'):
                continue
            
            output_name = template_file.replace('.j2', '')
            output_path = output_dir / output_name
            
            rendered = self.render(palette, template_file)
            output_path.write_text(rendered)
            
            results[template_file] = output_path
        
        return results
    
    def available_templates(self) -> list[str]:
        """List available templates."""
        return [t for t in self.env.list_templates() if t.endswith('.j2')]
````

**Tasks:**
- [ ] Implement `ThemeRenderer` class
- [ ] Implement `render()` method
- [ ] Implement `render_all()` method
- [ ] Implement `available_templates()` method
- [ ] Add error handling (template errors, file write errors)
- [ ] Write unit tests:
````python
  def test_render_kitty():
      renderer = ThemeRenderer()
      palette = load_test_palette()
      result = renderer.render(palette, 'kitty.conf.j2')
      assert 'background' in result
      assert 'color0' in result
````

---

### Expand CLI Interface

**Add to** `faelight_theme/cli.py`:
````python
from .render import ThemeRenderer

@cli.command()
@click.argument('palette_file', type=click.Path(exists=True))
@click.option('--target', help='Specific template (kitty, waybar, etc.)')
@click.option('--all', 'render_all', is_flag=True, help='Render all templates')
@click.option('--output-dir', type=click.Path(), help='Output directory')
@click.option('--output', '-o', type=click.Path(), help='Output file (single template)')
@click.option('--list', 'list_templates', is_flag=True, help='List available templates')
def render(palette_file, target, render_all, output_dir, output, list_templates):
    """Render configuration templates from palette."""
    
    renderer = ThemeRenderer()
    
    # List templates
    if list_templates:
        click.echo("Available templates:")
        for template in renderer.available_templates():
            click.echo(f"  - {template}")
        return
    
    # Load palette
    with open(palette_file) as f:
        palette = json.load(f)
    
    # Render all
    if render_all:
        if not output_dir:
            click.echo("Error: --output-dir required with --all", err=True)
            return
        
        results = renderer.render_all(palette, output_dir)
        click.echo(f"✅ Rendered {len(results)} templates to {output_dir}")
        for template, path in results.items():
            click.echo(f"  - {template} → {path}")
        return
    
    # Render specific template
    if not target:
        click.echo("Error: --target or --all required", err=True)
        return
    
    template_name = f"{target}.j2" if not target.endswith('.j2') else target
    result = renderer.render(palette, template_name)
    
    if output:
        Path(output).write_text(result)
        click.echo(f"✅ Rendered to {output}")
    else:
        click.echo(result)

@cli.command()
def list_templates():
    """List available templates."""
    renderer = ThemeRenderer()
    click.echo("Available templates:")
    for template in renderer.available_templates():
        name = template.replace('.j2', '')
        click.echo(f"  - {name}")
````

**Tasks:**
- [ ] Implement `render` command
- [ ] Add `--target` option for specific templates
- [ ] Add `--all` flag for batch rendering
- [ ] Add `--output-dir` option
- [ ] Add `--list` flag to show available templates
- [ ] Test CLI:
````bash
  faelight-theme render palette.json --target kitty
  faelight-theme render palette.json --all --output-dir ./generated
  faelight-theme list-templates
````

---

### Validation & Testing

**Config Validation:**

**Tasks:**
- [ ] Test Kitty config loads: `kitty --config generated/kitty.conf --version`
- [ ] Test Waybar CSS is valid: `killall waybar && waybar -c generated/waybar.css`
- [ ] Test Hyprland config syntax
- [ ] Create validation functions for each config type
- [ ] Add to `dot-doctor` integration (future)

**Comprehensive Testing:**

**Tasks:**
- [ ] Test with dark palette
- [ ] Test with light palette
- [ ] Test with vibrant palette
- [ ] Test with muted palette
- [ ] Verify all variables are replaced
- [ ] Verify no Jinja2 syntax remains in output
- [ ] Run full test suite: `pytest tests/`

---

### Documentation

**Tasks:**
- [ ] Update `README.md` with render examples
- [ ] Document template system architecture
- [ ] Document template variables
- [ ] Create template development guide
- [ ] Document how to add new templates
- [ ] Add rendered examples to `examples/`

---

**Estimated Time:** 4-5 hours  
**Dependencies:** v2.8.2 complete ✅  
**Deliverables:**
- ✅ Jinja2 template system implemented
- ✅ 5+ config templates created
- ✅ Working `faelight-theme render` command
- ✅ All templates tested and validated
- ✅ **Standalone theme engine COMPLETE!**

**Next:** v2.8.4 integrates with FCM/dotfiles!

---

## 🎨 Version 2.8.4 - FCM Integration (Consumer Layer)

### Goals: Make FCM Consume Standalone Theme Engine

**Work Location:** `~/dotfiles/` (back to dotfiles repo!)

**Architecture:**
````
faelight-theme-engine (standalone Python package)
         ↓ CLI API
FCM theme-from-wallpaper.sh (consumer script)
         ↓ Generates
Atomic wallpaper packages (waybar-theme-wallpaper, kitty-theme-wallpaper)
         ↓ Installed by
Stow (or dot-apply in v3.0)
````

**Workflow:**
````
Install Engine → Create Script → Create Atomic Packages → Test → Integrate
````

---

### Install Theme Engine as Dependency

**Tasks:**
- [ ] Install in editable mode: `pip install -e /path/to/faelight-theme-engine`
- [ ] Verify CLI works: `faelight-theme --version`
- [ ] Add to dotfiles documentation:
````markdown
  ## Dependencies
  
  ### Faelight Theme Engine
  Required for wallpaper-based theme generation.
```bash
  git clone https://github.com/yourusername/faelight-theme-engine
  pip install -e ./faelight-theme-engine
```
````
- [ ] Document in `COMPLETE_GUIDE.md`

---

### Create Integration Script

**File:** `~/dotfiles/scripts/theme-from-wallpaper.sh`
````bash
#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════
# 🎨 Faelight Theme - Generate from Wallpaper
# Uses: faelight-theme-engine (standalone)
# ═══════════════════════════════════════════════════════════

set -e

WALLPAPER="$1"
DOTFILES="$HOME/dotfiles"
TEMP_DIR="/tmp/faelight-generated-$(date +%s)"

# Check dependencies
if ! command -v faelight-theme &> /dev/null; then
    echo "❌ Error: faelight-theme not found"
    echo "Install: pip install -e /path/to/faelight-theme-engine"
    exit 1
fi

if [[ -z "$WALLPAPER" ]]; then
    echo "Usage: theme-from-wallpaper.sh <wallpaper-path>"
    echo ""
    echo "Example:"
    echo "  theme-from-wallpaper.sh ~/Pictures/Wallpapers/forest.jpg"
    exit 1
fi

if [[ ! -f "$WALLPAPER" ]]; then
    echo "❌ Error: Wallpaper not found: $WALLPAPER"
    exit 1
fi

echo "🎨 Faelight Theme Generator"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Wallpaper: $WALLPAPER"
echo ""

# 1. Extract palette using theme engine
echo "🔍 Extracting colors..."
PALETTE_FILE="$TEMP_DIR/palette.json"
mkdir -p "$TEMP_DIR"
faelight-theme extract "$WALLPAPER" --output "$PALETTE_FILE"

if [[ ! -f "$PALETTE_FILE" ]]; then
    echo "❌ Failed to extract colors"
    exit 1
fi

echo "✅ Colors extracted"
echo ""

# 2. Render all configs using theme engine
echo "📝 Rendering configurations..."
faelight-theme render "$PALETTE_FILE" --all --output-dir "$TEMP_DIR"

echo "✅ Configurations rendered"
echo ""

# 3. Copy to atomic wallpaper packages in dotfiles
echo "📦 Installing theme packages..."

# Kitty
mkdir -p "$DOTFILES/kitty-theme-wallpaper/.config/kitty"
cp "$TEMP_DIR/kitty.conf" "$DOTFILES/kitty-theme-wallpaper/.config/kitty/current-theme.conf"
echo "  ✅ kitty-theme-wallpaper"

# Waybar
mkdir -p "$DOTFILES/waybar-theme-wallpaper/.config/waybar"
cp "$TEMP_DIR/waybar-style.css" "$DOTFILES/waybar-theme-wallpaper/.config/waybar/style.css"
echo "  ✅ waybar-theme-wallpaper"

# Hyprland
mkdir -p "$DOTFILES/hypr-theme-wallpaper/.config/hypr"
cp "$TEMP_DIR/hyprland-colors.conf" "$DOTFILES/hypr-theme-wallpaper/.config/hypr/colors.conf"
echo "  ✅ hypr-theme-wallpaper"

# Mako
mkdir -p "$DOTFILES/mako-theme-wallpaper/.config/mako"
cp "$TEMP_DIR/mako.conf" "$DOTFILES/mako-theme-wallpaper/.config/mako/config"
echo "  ✅ mako-theme-wallpaper"

echo ""

# 4. Apply using Stow (or dot-apply when v3.0 is ready)
echo "🔄 Applying theme..."
cd "$DOTFILES"

# Unstow current wallpaper theme if active
stow -D kitty-theme-wallpaper 2>/dev/null || true
stow -D waybar-theme-wallpaper 2>/dev/null || true
stow -D hypr-theme-wallpaper 2>/dev/null || true
stow -D mako-theme-wallpaper 2>/dev/null || true

# Stow new wallpaper theme
stow kitty-theme-wallpaper
stow waybar-theme-wallpaper
stow hypr-theme-wallpaper
stow mako-theme-wallpaper

echo "✅ Theme installed"
echo ""

# 5. Reload applications
echo "✨ Reloading applications..."
hyprctl reload
killall -SIGUSR1 kitty 2>/dev/null || true
killall waybar 2>/dev/null && waybar & disown
makoctl reload

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Theme applied successfully!"
echo "🎨 Source: $(basename "$WALLPAPER")"
echo ""
echo "💡 To revert to manual theme:"
echo "   theme-switch dark  (or light)"

# Cleanup
rm -rf "$TEMP_DIR"
````

**Tasks:**
- [ ] Create `scripts/` directory if needed
- [ ] Create `theme-from-wallpaper.sh` script
- [ ] Make executable: `chmod +x scripts/theme-from-wallpaper.sh`
- [ ] Add to PATH or create alias:
````fish
  alias theme-wall='~/dotfiles/scripts/theme-from-wallpaper.sh'
  alias theme-gen='~/dotfiles/scripts/theme-from-wallpaper.sh'
````
- [ ] Test with sample wallpaper

---

### Create Atomic Wallpaper Packages

**Package Structure:**
````
dotfiles/
├── kitty-theme-wallpaper/
│   └── .config/kitty/
│       └── current-theme.conf
├── waybar-theme-wallpaper/
│   └── .config/waybar/
│       └── style.css
├── hypr-theme-wallpaper/
│   └── .config/hypr/
│       └── colors.conf
└── mako-theme-wallpaper/
    └── .config/mako/
        └── config
````

**Tasks:**
- [ ] Create empty directory structures
- [ ] Add placeholder files (will be overwritten by script)
- [ ] Add `.stow-local-ignore` if needed (to ignore temp files)
- [ ] Test stowing/unstowing:
````bash
  cd ~/dotfiles
  stow kitty-theme-wallpaper
  ls -la ~/.config/kitty/current-theme.conf  # Should be symlink
  stow -D kitty-theme-wallpaper
````
- [ ] Verify symlinks point to correct locations

---

### Application Integration & Reload Logic

**Hyprland Integration:**

**Tasks:**
- [ ] Verify Hyprland sources colors.conf:
````conf
  # In hypr/.config/hypr/hyprland.conf
  source = ~/.config/hypr/colors.conf
````
- [ ] Test reload: `hyprctl reload`
- [ ] Verify colors change

**Kitty Integration:**

**Tasks:**
- [ ] Verify Kitty includes current-theme.conf:
````conf
  # In kitty/.config/kitty/kitty.conf
  include current-theme.conf
````
- [ ] Test reload: `killall -SIGUSR1 kitty`
- [ ] Verify terminal colors change instantly

**Waybar Integration:**

**Tasks:**
- [ ] Verify Waybar uses style.css
- [ ] Test reload: `killall waybar && waybar &`
- [ ] Verify bar colors change

**Mako Integration:**

**Tasks:**
- [ ] Verify Mako uses config
- [ ] Test reload: `makoctl reload`
- [ ] Send test notification, verify colors

---

### Integration with Existing Theme System

**Tasks:**
- [ ] Integrate with `theme-switch.sh`:
````bash
  # Add to theme-switch.sh
  case "$1" in
      dark|light)
          # Existing logic
          ;;
      wallpaper)
          # Call theme-from-wallpaper.sh
          ;;
  esac
````
- [ ] Update theme tracking (if using current.txt)
- [ ] Ensure wallpaper theme can coexist with dark/light themes

---

### Testing Phase

**Tasks:**
- [ ] Test with dark wallpaper:
````bash
  theme-from-wallpaper.sh ~/Pictures/dark-wallpaper.jpg
  # Verify: dark background, light text, good contrast
````
- [ ] Test with light wallpaper:
````bash
  theme-from-wallpaper.sh ~/Pictures/light-wallpaper.jpg
  # Verify: light background, dark text, good contrast
````
- [ ] Test with colorful wallpaper:
````bash
  theme-from-wallpaper.sh ~/Pictures/colorful-wallpaper.jpg
  # Verify: vibrant colors, good accents
````
- [ ] Test with muted wallpaper:
````bash
  theme-from-wallpaper.sh ~/Pictures/muted-wallpaper.jpg
  # Verify: subtle colors, calm aesthetic
````
- [ ] Test with gradient wallpaper:
````bash
  theme-from-wallpaper.sh ~/Pictures/gradient-wallpaper.jpg
  # Verify: smooth color transitions
````
- [ ] Verify all applications reload correctly
- [ ] Check for visual glitches
- [ ] Run `dot-doctor` after each application
- [ ] Test theme switching: dark → wallpaper → light → wallpaper

---

### Documentation

**Tasks:**
- [ ] Create `THEME_ENGINE.md` guide:
  - How to use theme-from-wallpaper.sh
  - How it works (architecture diagram)
  - Customization options
  - Troubleshooting
- [ ] Update `COMPLETE_GUIDE.md`:
  - Add theme engine section
  - Document all commands
  - Show examples
- [ ] Update `README.md`:
  - Add theme engine to features
  - Show screenshot examples (before/after)
- [ ] Add to `CHANGELOG.md`
- [ ] Create example screenshots

---

**Estimated Time:** 5-6 hours  
**Dependencies:** v2.8.3 complete ✅  
**Deliverables:**
- ✅ FCM consumes standalone theme engine via CLI
- ✅ `theme-from-wallpaper.sh` working perfectly
- ✅ Atomic wallpaper packages created
- ✅ All applications integrate correctly
- ✅ Complete documentation
- ✅ **Wallpaper-based themes now USABLE!** 🎨

**Next:** v2.8.5 adds polish, caching, and automation!

---

## 🎨 Version 2.8.5 - Polish, Features & Automation

### Goals: Add Intelligence & User Experience Features

**Workflow:**
````
Automate → Cache → Preview → Polish → Ship
````

---

### Wallpaper Change Detection & Auto-Apply

**Goal:** Automatically regenerate theme when wallpaper changes

**Implementation:** `~/dotfiles/scripts/wallpaper-watcher.sh`
````bash
#!/usr/bin/env bash
# Watch for wallpaper changes and auto-apply theme

WALLPAPER_FILE="$HOME/.config/hypr/hyprpaper.conf"
LAST_WALLPAPER=""

while true; do
    # Extract current wallpaper from hyprpaper config
    CURRENT=$(grep "^preload" "$WALLPAPER_FILE" | head -1 | awk '{print $3}')
    
    if [[ "$CURRENT" != "$LAST_WALLPAPER" ]] && [[ -n "$CURRENT" ]]; then
        echo "🎨 Wallpaper changed: $CURRENT"
        echo "Generating theme..."
        ~/dotfiles/scripts/theme-from-wallpaper.sh "$CURRENT"
        LAST_WALLPAPER="$CURRENT"
    fi
    
    sleep 5
done
````

**Tasks:**
- [ ] Create wallpaper-watcher.sh script
- [ ] Test wallpaper change detection
- [ ] Add systemd user service (optional):
````ini
  [Unit]
  Description=Faelight Wallpaper Theme Watcher
  
  [Service]
  ExecStart=/home/christian/dotfiles/scripts/wallpaper-watcher.sh
  Restart=always
  
  [Install]
  WantedBy=default.target
````
- [ ] Add to startup (optional)
- [ ] Document in THEME_ENGINE.md

---

### Theme Caching System

**Goal:** Skip regeneration if wallpaper unchanged

**Implementation:** Add to `theme-from-wallpaper.sh`
````bash
# After extracting wallpaper path
WALLPAPER_HASH=$(md5sum "$WALLPAPER" | awk '{print $1}')
CACHE_DIR="$HOME/.cache/faelight-themes"
CACHED_PALETTE="$CACHE_DIR/$WALLPAPER_HASH.json"

mkdir -p "$CACHE_DIR"

# Check cache
if [[ -f "$CACHED_PALETTE" ]]; then
    echo "✅ Using cached palette"
    PALETTE_FILE="$CACHED_PALETTE"
else
    echo "🔍 Extracting colors..."
    faelight-theme extract "$WALLPAPER" --output "$CACHED_PALETTE"
    PALETTE_FILE="$CACHED_PALETTE"
fi
````

**Tasks:**
- [ ] Implement caching in theme-from-wallpaper.sh
- [ ] Store generated palettes by hash
- [ ] Add cache invalidation (--force flag)
- [ ] Add cache cleanup command:
````fish
  function theme-clean-cache
      rm -rf ~/.cache/faelight-themes/*
      echo "✅ Cache cleared"
  end
````
- [ ] Test cache hits/misses
- [ ] Document cache location

---

### Live Preview System

**Goal:** Preview theme before applying

**Implementation:** Add `--preview` flag to theme-from-wallpaper.sh
````bash
# Add flag parsing
PREVIEW_MODE=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --preview)
            PREVIEW_MODE=true
            shift
            ;;
        *)
            WALLPAPER="$1"
            shift
            ;;
    esac
done

# After rendering configs
if [[ "$PREVIEW_MODE" == "true" ]]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🎨 Theme Preview"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Show palette (using jq)
    jq -r '. | "Background: \(.background)\nForeground: \(.foreground)\nAccent: \(.accent)"' "$PALETTE_FILE"
    
    # Show terminal color preview
    faelight-theme preview "$PALETTE_FILE"
    
    echo ""
    read -p "Apply this theme? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Theme not applied"
        exit 0
    fi
fi
````

**Tasks:**
- [ ] Add `--preview` flag to script
- [ ] Implement palette display (jq formatting)
- [ ] Add to theme engine: `faelight-theme preview palette.json`
  - Show colors in terminal
  - Use ANSI escape codes to display actual colors
- [ ] Add confirmation prompt
- [ ] Test preview workflow
- [ ] Document preview mode

---

### Palette Generation Presets

**Goal:** Control theme generation style

**Add to faelight-theme engine** (`faelight_theme/palette.py`):
````python
def generate_palette(
    base_colors: list,
    preset: str = "balanced"
) -> dict:
    """
    Generate palette with style preset.
    
    Presets:
        - balanced: Default, natural colors
        - vibrant: Boost saturation +20%
        - muted: Reduce saturation -20%
        - pastel: Light, soft colors
        - dark: Force dark background
        - light: Force light background
    """
    # Apply preset transformations
    if preset == "vibrant":
        base_colors = [increase_saturation(c, 1.2) for c in base_colors]
    elif preset == "muted":
        base_colors = [decrease_saturation(c, 0.8) for c in base_colors]
    # ... etc
````

**Tasks:**
- [ ] Implement preset system in theme engine
- [ ] Add `--preset` option to CLI:
````bash
  faelight-theme extract wallpaper.jpg --preset vibrant
````
- [ ] Add to theme-from-wallpaper.sh:
````bash
  theme-from-wallpaper.sh wallpaper.jpg --preset muted
````
- [ ] Test all presets
- [ ] Document presets in THEME_ENGINE.md

---

### Wallpaper Gallery & Favorites

**Goal:** Manage wallpapers and generated themes

**Implementation:** `~/dotfiles/scripts/theme-gallery.sh`
````bash
#!/usr/bin/env bash
# Browse wallpapers and their generated themes

WALLPAPER_DIR="$HOME/Pictures/Wallpapers"
CACHE_DIR="$HOME/.cache/faelight-themes"

echo "🖼️  Wallpaper Gallery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for wallpaper in "$WALLPAPER_DIR"/*.{jpg,jpeg,png}; do
    [[ -f "$wallpaper" ]] || continue
    
    hash=$(md5sum "$wallpaper" | awk '{print $1}')
    cached="$CACHE_DIR/$hash.json"
    
    name=$(basename "$wallpaper")
    
    if [[ -f "$cached" ]]; then
        echo "✅ $name (theme cached)"
    else
        echo "⚪ $name (no theme)"
    fi
done

echo ""
echo "Commands:"
echo "  theme-from-wallpaper.sh <wallpaper>  - Generate theme"
echo "  theme-from-wallpaper.sh <wallpaper> --preview  - Preview first"
````

**Tasks:**
- [ ] Create theme-gallery.sh script
- [ ] List wallpapers with cache status
- [ ] Add favorites system (symlinks or database)
- [ ] Add preview-all command
- [ ] Document in THEME_ENGINE.md

---

### Fish Shell Integration

**Add aliases to** `~/dotfiles/fish/.config/fish/config.fish`:
````fish
# ═══════════════════════════════════════════════════════════
# 🎨 Theme Engine Aliases
# ═══════════════════════════════════════════════════════════

# Quick theme generation
alias theme-gen='~/dotfiles/scripts/theme-from-wallpaper.sh'
alias theme-wall='~/dotfiles/scripts/theme-from-wallpaper.sh'

# With preview
alias theme-preview='~/dotfiles/scripts/theme-from-wallpaper.sh --preview'

# Gallery
alias theme-gallery='~/dotfiles/scripts/theme-gallery.sh'

# Cache management
function theme-clean-cache --description "Clear theme cache"
    rm -rf ~/.cache/faelight-themes/*
    echo "✅ Theme cache cleared"
end

function theme-list-cache --description "List cached themes"
    ls -lh ~/.cache/faelight-themes/
end
````

**Tasks:**
- [ ] Add aliases to fish config
- [ ] Test all aliases
- [ ] Update Fish completion (optional)
- [ ] Document in COMPLETE_GUIDE.md

---

### Documentation & Polish

**Tasks:**
- [ ] Complete THEME_ENGINE.md comprehensive guide:
  - Installation
  - Basic usage
  - Advanced features
  - Presets explained
  - Caching explained
  - Troubleshooting
  - Tips and tricks
- [ ] Update COMPLETE_GUIDE.md:
  - Add theme engine chapter
  - Document all commands
  - Show workflow examples
- [ ] Create video/GIF demos:
  - Basic theme generation
  - Preview mode
  - Automatic wallpaper detection
- [ ] Add to README.md showcase
- [ ] Update CHANGELOG.md

---

**Estimated Time:** 5-6 hours  
**Dependencies:** v2.8.4 complete ✅  
**Deliverables:**
- ✅ Wallpaper change detection (optional automation)
- ✅ Theme caching (instant reapplication)
- ✅ Live preview mode
- ✅ Preset system (vibrant, muted, pastel, etc.)
- ✅ Wallpaper gallery
- ✅ Complete Fish integration
- ✅ Comprehensive documentation
- ✅ **Theme Engine fully featured and polished!** 🎨✨

**Next:** v2.8.6 enhances diagnostic tooling!

---

## 🔧 Version 2.8.6 - Advanced Dotfile Intelligence Suite

### Goals: Enhance Existing Tooling

**Workflow:**
````
Enhance dot-doctor → Enhance keyscan → Add New Tools → Automate → Document
````

---

### Enhanced dot-doctor

**Add Advanced Checks:**

**Tasks:**
- [ ] Font dependency verification:
````fish
  function _check_fonts
      # Check for required fonts
      fc-list | grep -i "Hack Nerd Font" > /dev/null && echo "✅ Hack Nerd Font" || echo "❌ Hack Nerd Font missing"
      fc-list | grep -i "Inter" > /dev/null && echo "✅ Inter" || echo "⚠️  Inter missing"
  end
````
- [ ] Orphaned file detection:
````fish
  function _check_orphans
      # Find configs without symlinks
      # Find old backup files
      # Report orphaned directories
  end
````
- [ ] Theme consistency checker:
````fish
  function _check_theme_consistency
      # Verify all theme files exist
      # Check dark theme complete
      # Check light theme complete
      # Check wallpaper theme if generated
  end
````
- [ ] Keybinding integration:
````fish
  function _check_keybindings
      # Call keyscan
      # Report if conflicts found
  end
````
- [ ] Git status check:
````fish
  function _check_git_status
      cd ~/dotfiles
      if test (git status --porcelain | wc -l) -gt 0
          echo "⚠️  Uncommitted changes in dotfiles"
      end
  end
````

**Add --fix Flag:**

**Tasks:**
- [ ] Implement auto-repair mode
- [ ] Recreate broken symlinks
- [ ] Install missing fonts (with confirmation)
- [ ] Remove orphaned files (with confirmation)
- [ ] Document what --fix does

**Add --report Flag:**

**Tasks:**
- [ ] Generate HTML health report
- [ ] Include graphs (using plotly or matplotlib)
- [ ] Email option (using mailx or similar)
- [ ] Save to ~/dotfiles/reports/

---

### Enhanced keyscan

**Expand Parsing:**

**Tasks:**
- [ ] Parse Waybar on-click handlers:
````fish
  function _parse_waybar_bindings
      # Read waybar config.jsonc
      # Extract on-click, on-scroll handlers
      # Report any that might conflict
  end
````
- [ ] Parse Kitty keybinds:
````fish
  function _parse_kitty_bindings
      # Read kitty.conf
      # Extract map bindings
      # Check for conflicts with Hyprland
  end
````
- [ ] Parse Fish key bindings (if any):
````fish
  function _parse_fish_bindings
      # Check for bind commands in config.fish
  end
````

**Beautiful Output:**

**Tasks:**
- [ ] Add color-coded categories (use set_color)
- [ ] Add ASCII art boxes (using unicode box drawing)
- [ ] Export to markdown table:
````fish
  function keyscan --argument --export
      # Generate KEYBINDINGS.md
  end
````

**Danger Zone Warnings:**

**Tasks:**
- [ ] Detect terminal vs system conflicts:
````
  ⚠️  DANGER: SUPER+C conflicts between:
     - Hyprland: VSCode
     - Kitty: Copy (Ctrl+Shift+C recommended instead)
````
- [ ] Detect common application conflicts
- [ ] Suggest alternatives

**Auto-Generate Documentation:**

**Tasks:**
- [ ] Add `keyscan --export` command
- [ ] Generate `docs/KEYBINDINGS.md` automatically
- [ ] Include descriptions from bindd
- [ ] Group by category
- [ ] Keep updated

---

### Additional Tools

**1. dot-diff - Visual Diff:**
````fish
function dot-diff --argument package
    # Compare current config vs dotfiles version
    if command -v meld &> /dev/null
        meld ~/.config/$package ~/dotfiles/$package/.config/$package
    else
        diff -r ~/.config/$package ~/dotfiles/$package/.config/$package
    end
end
````

**2. dot-benchmark - Performance Profiling:**
````fish
function dot-benchmark
    echo "📊 Performance Benchmark"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Fish startup time
    set -l fish_time (time fish -c exit 2>&1 | grep real | awk '{print $2}')
    echo "Fish startup: $fish_time"
    
    # Hyprland startup (from logs)
    echo "Hyprland startup: (check logs)"
    
    # Plugin load times
    echo "Analyzing plugins..."
end
````

**3. dot-update - Safe Update Workflow:**
````fish
function dot-update
    echo "🔄 Safe Dotfile Update Workflow"
    
    # 1. Snapshot
    dot-snapshot "Pre-update $(date)"
    
    # 2. Run dot-doctor
    dot-doctor || return 1
    
    # 3. Update system
    sudo pacman -Syu
    
    # 4. Run dot-doctor again
    dot-doctor
    
    # 5. Update dotfiles
    cd ~/dotfiles
    git pull
    
    # 6. Final check
    dot-doctor
end
````

**4. theme-validate - Validate Generated Themes:**
````fish
function theme-validate
    echo "🎨 Theme Validation"
    
    # Check contrast ratios using faelight-theme
    faelight-theme validate ~/.cache/faelight-themes/latest.json
    
    # Verify all configs exist
    # Run syntax checks
end
````

**Tasks:**
- [ ] Implement all 4 tools
- [ ] Write Fish functions
- [ ] Test each tool
- [ ] Add to config.fish
- [ ] Document in TOOLING.md

---

### Integration & Automation

**Pre-commit Hook (Optional):**

**File:** `~/dotfiles/.git/hooks/pre-commit`
````bash
#!/bin/bash
# Validate dotfiles before commit

echo "Running dot-doctor..."
fish -c dot-doctor || exit 1

echo "Running keyscan..."
fish -c "keyscan | grep 'Conflicts: 0'" || exit 1

echo "✅ All checks passed"
````

**Tasks:**
- [ ] Create pre-commit hook
- [ ] Make executable
- [ ] Test by making bad commit
- [ ] Document how to enable/disable

**Weekly Health Check Reminder:**

**Tasks:**
- [ ] Add to cron (optional):
````cron
  0 9 * * SUN /home/christian/dotfiles/scripts/weekly-health-check.sh
````
- [ ] Create weekly-health-check.sh script
- [ ] Email/notify if issues found
- [ ] Document in TOOLING.md

**Health Command Integration:**

**Tasks:**
- [ ] Update `health` alias to include dot-doctor summary
- [ ] Show last health check time
- [ ] Show any warnings

---

### Documentation

**Tasks:**
- [ ] Update TOOLING.md comprehensively:
  - All commands explained with examples
  - Usage patterns
  - Troubleshooting section
- [ ] Update COMPLETE_GUIDE.md:
  - Add "Advanced Tooling" section
  - Document weekly maintenance routine with new tools
- [ ] Create troubleshooting flowcharts (Mermaid diagrams)
- [ ] Document all aliases and functions
- [ ] Add to CHANGELOG.md

---

**Estimated Time:** 4-5 hours  
**Dependencies:** v2.8.5 complete ✅  
**Priority:** Medium (polish after theme engine)  
**Deliverables:**
- ✅ Enhanced dot-doctor with --fix and --report
- ✅ Enhanced keyscan with exports and warnings
- ✅ 4 new diagnostic tools
- ✅ Optional automation (pre-commit, cron)
- ✅ Complete professional tooling suite
- ✅ Comprehensive documentation

**Next:** v2.9 - Security & Backup Infrastructure!

---

## 📊 Version 2.8 - Summary

**Complete v2.8 Structure:**
````
v2.8.0 - Foundational Intelligence (COMPLETE ✅)
├─ dot-doctor (basic health monitoring)
├─ keyscan (conflict detection)
└─ Safety baseline established

v2.8.1 - Theme Engine Foundation (3-4 hours)
├─ Standalone Python package structure
├─ Research & design decisions
└─ Ready for implementation

v2.8.2 - Color Extraction (4-5 hours)
├─ Extraction algorithm
├─ Palette generation
├─ WCAG validation
└─ CLI implementation

v2.8.3 - Template System (4-5 hours)
├─ Jinja2 templates
├─ Rendering engine
└─ Complete standalone theme engine

v2.8.4 - FCM Integration (5-6 hours)
├─ FCM consumes theme engine
├─ Atomic wallpaper packages
├─ Application integration
└─ Theme system operational

v2.8.5 - Polish & Features (5-6 hours)
├─ Caching system
├─ Preview mode
├─ Presets (vibrant, muted, etc.)
├─ Gallery
└─ Full automation

v2.8.6 - Advanced Tooling (4-5 hours)
├─ Enhanced dot-doctor
├─ Enhanced keyscan
├─ Additional tools
└─ Professional polish
````

**Total Time:** 28-34 hours (spread over weeks/months)

**Key Architectural Decisions:**
- ✅ Theme engine is STANDALONE (reusable, testable)
- ✅ Clean CLI API (FCM is consumer)
- ✅ Separation of concerns (theme gen vs config management)

**After v2.8 Complete:**
Your system will generate beautiful themes from ANY wallpaper, with professional tooling to monitor health and prevent conflicts! 🎨🛡️🌲

---

## 🔐 Version 2.9 - Security & Backup Infrastructure

### Goals: Advanced Security + Complete Backup Strategy

**Total Estimated Time:** 6-8 hours

*(Keep existing v2.9 structure - no changes needed)*

---

## ⚛️ Version 3.0 - Faelight Config Manager Foundation

### Overview: Atomic Packages + Declarative Configuration

**CRITICAL:** Atomic package restructure is **MANDATORY** for v3.0. This is the foundation for everything else.

**Core Architecture:**
````
Atomic Packages (base + theme + machine)
         ↓
Package Metadata (.dotfile-meta.yaml)
         ↓
Python Dependency Resolver (networkx)
         ↓
Declarative Manifest (manifest.yaml)
         ↓
Git Tag Snapshots (not YAML files)
````

**Total Estimated Time:** 20-25 hours (spread over 2-3 weeks)

---

### Phase 1: Atomic Package Restructure (8-10 hours) **MANDATORY**

**Goal:** Break monolithic packages into composable units

**This is NON-NEGOTIABLE - everything else depends on this!**

**Current Structure (Monolithic):**
````
dotfiles/
├── fish/           # Config + aliases + functions + theme (all mixed)
├── hypr/           # Config + theme + machine-specific (all mixed)
└── waybar/         # Structure + styling + modules (all mixed)
````

**Target Structure (Atomic):**
````
dotfiles/
├── fish-base/              # Core config only
├── fish-aliases/           # Just aliases
├── fish-functions/         # Just functions  
├── fish-theme-dark/        # Dark prompt colors
├── fish-theme-light/       # Light prompt colors
├── fish-theme-wallpaper/   # Generated theme (from v2.8)

├── hypr-base/              # Core Hyprland config
├── hypr-laptop/            # Laptop-specific (battery, backlight)
├── hypr-theme-dark/        # Dark window colors
├── hypr-theme-light/       # Light window colors
├── hypr-theme-wallpaper/   # Generated colors (from v2.8)

├── waybar-base/            # Structure + modules
├── waybar-laptop/          # Laptop modules (battery, backlight)
├── waybar-theme-dark/      # Dark CSS
├── waybar-theme-light/     # Light CSS
├── waybar-theme-wallpaper/ # Generated CSS (from v2.8)

├── kitty-base/             # Core Kitty config
├── kitty-theme-dark/       # Dark terminal.conf
├── kitty-theme-light/      # Light terminal.conf
├── kitty-theme-wallpaper/  # Generated theme (from v2.8)

├── nvim/                   # Keep monolithic (LazyVim manages itself)
├── yazi/                   # Keep monolithic (simple)
├── mako/                   # Keep monolithic
└── gtk/                    # Keep monolithic
````

**Why Atomic Packages:**
1. **Composability** - Mix and match (base + dark + laptop)
2. **Theme Switching** - Just swap theme packages
3. **Machine Profiles** - Different laptop/desktop configs
4. **Testing** - Test packages in isolation
5. **Clarity** - Clean separation of concerns

---

#### Step 1A: Plan the Split (1 hour)

**Tasks:**
- [ ] Document current fish package contents
- [ ] Document current hypr package contents
- [ ] Document current waybar package contents
- [ ] Document current kitty package contents
- [ ] Create splitting strategy document
- [ ] Identify which files go in which atomic package
- [ ] Plan migration approach (all at once vs incremental)

---

#### Step 1B: Split Fish Package (2-3 hours)

**Tasks:**
- [ ] Create new atomic package directories:
````bash
  mkdir -p fish-base/.config/fish
  mkdir -p fish-aliases/.config/fish/conf.d
  mkdir -p fish-functions/.config/fish/functions
  mkdir -p fish-theme-dark/.config/fish/conf.d
  mkdir -p fish-theme-light/.config/fish/conf.d
````
- [ ] Move core config.fish to fish-base/
  - Remove aliases (go to fish-aliases)
  - Remove functions (go to fish-functions)
  - Remove theme colors (go to fish-theme-*)
  - Keep only core settings
- [ ] Move all aliases to fish-aliases/
  - Create aliases.fish
  - Source from conf.d/
- [ ] Move all functions to fish-functions/
  - One file per function
  - Place in functions/ directory
- [ ] Extract theme colors to fish-theme-dark/ and fish-theme-light/
  - Create theme-colors.fish for each
  - Include prompt customizations
- [ ] Test each atomic package individually:
````bash
  cd ~/dotfiles
  stow fish-base
  stow fish-aliases
  stow fish-functions
  stow fish-theme-dark
  # Test fish loads without errors
````
- [ ] Verify all combinations work
- [ ] Update .stow-local-ignore if needed

---

#### Step 1C: Split Hypr Package (2-3 hours)

**Tasks:**
- [ ] Create atomic package directories:
````bash
  mkdir -p hypr-base/.config/hypr
  mkdir -p hypr-laptop/.config/hypr
  mkdir -p hypr-theme-dark/.config/hypr
  mkdir -p hypr-theme-light/.config/hypr
````
- [ ] Split hyprland.conf into:
  - hypr-base/ - Core config (no colors, no machine-specific)
  - hypr-laptop/ - Battery, backlight, laptop-specific settings
  - hypr-theme-dark/ - colors.conf with dark colors
  - hypr-theme-light/ - colors.conf with light colors
- [ ] Ensure hypr-base sources colors.conf:
````conf
  source = ~/.config/hypr/colors.conf
````
- [ ] Test each package:
````bash
  stow hypr-base hypr-laptop hypr-theme-dark
  hyprctl reload
  # Verify no errors
````
- [ ] Test all combinations:
  - base + laptop + dark
  - base + laptop + light
  - base + laptop + wallpaper (from v2.8)

---

#### Step 1D: Split Waybar Package (2-3 hours)

**Tasks:**
- [ ] Create atomic package directories:
````bash
  mkdir -p waybar-base/.config/waybar
  mkdir -p waybar-laptop/.config/waybar
  mkdir -p waybar-theme-dark/.config/waybar
  mkdir -p waybar-theme-light/.config/waybar
````
- [ ] Split into:
  - waybar-base/ - config.jsonc structure + base modules
  - waybar-laptop/ - Battery, backlight, network modules
  - waybar-theme-dark/ - style-dark.css
  - waybar-theme-light/ - style-light.css
- [ ] Ensure config.jsonc imports style.css:
````json
  "style": "~/.config/waybar/style.css"
````
- [ ] Test combinations
- [ ] Verify bar displays correctly

---

#### Step 1E: Split Kitty Package (1-2 hours)

**Tasks:**
- [ ] Create atomic package directories:
````bash
  mkdir -p kitty-base/.config/kitty
  mkdir -p kitty-theme-dark/.config/kitty
  mkdir -p kitty-theme-light/.config/kitty
````
- [ ] Split kitty.conf:
  - kitty-base/ - Core config (no colors)
  - kitty-theme-dark/ - current-theme.conf dark
  - kitty-theme-light/ - current-theme.conf light
- [ ] Ensure kitty.conf includes theme:
````conf
  include current-theme.conf
````
- [ ] Test all combinations

---

#### Step 1F: Cleanup & Verification (1 hour)

**Tasks:**
- [ ] Remove old monolithic packages:
````bash
  # Unstow old packages
  stow -D fish hypr waybar kitty
  
  # Archive old structure (don't delete yet!)
  mv fish fish-OLD-BACKUP
  mv hypr hypr-OLD-BACKUP
  mv waybar waybar-OLD-BACKUP
  mv kitty kitty-OLD-BACKUP
````
- [ ] Test full system with atomic packages:
````bash
  stow fish-base fish-aliases fish-functions fish-theme-dark
  stow hypr-base hypr-laptop hypr-theme-dark
  stow waybar-base waybar-laptop waybar-theme-dark
  stow kitty-base kitty-theme-dark
  stow nvim yazi mako gtk
````
- [ ] Verify everything works
- [ ] Run dot-doctor
- [ ] Run keyscan
- [ ] Test for 1 week before deleting backups
- [ ] Once stable, delete -OLD-BACKUP directories

---

**Deliverables:**
- ✅ Fully atomic package structure
- ✅ Clean separation (base / theme / machine)
- ✅ All combinations tested
- ✅ Ready for metadata and dependency system

**This restructure is the FOUNDATION - don't skip it!**

---

### Phase 2: Package Metadata System (6-8 hours)

**Goal:** Add intelligence with metadata files

**Metadata Format:**
````yaml
# waybar-theme-dark/.dotfile-meta.yaml
name: waybar-theme-dark
version: 3.0.0
description: "Dark theme for Waybar with Faelight Forest colors"
author: Christian
created: 2025-12-03
updated: 2025-12-03

type: theme
category: waybar

depends:
  - waybar-base

conflicts:
  - waybar-theme-light
  - waybar-theme-wallpaper

provides:
  - waybar-theme-colors

files:
  generated: []
  handwritten:
    - .config/waybar/style-dark.css

required-binaries: []

required-fonts:
  - Hack Nerd Font
  - Inter

tags:
  - theme
  - dark
  - waybar
  - faelight-forest

compatibility:
  hyprland: ">=0.40.0"
  waybar: ">=0.10.0"
````

**Tasks:**
- [ ] Create `.dotfile-meta.yaml` template
- [ ] Write metadata for ALL atomic packages (30-40 files!)
  - [ ] All fish-* packages (6 packages)
  - [ ] All hypr-* packages (5 packages)
  - [ ] All waybar-* packages (5 packages)
  - [ ] All kitty-* packages (4 packages)
  - [ ] Remaining monolithic packages (nvim, yazi, etc.)
- [ ] Create validation script (YAML syntax checker)
- [ ] Create `dot-info` command (query metadata)
- [ ] Create `dot-search` command (find packages)
- [ ] Create `dot-list` command (list installed)
- [ ] Document metadata schema in FRAMEWORK.md

**Deliverables:**
- ✅ Every package has metadata
- ✅ Query/search tools working
- ✅ Foundation for dependencies

---

### Phase 3: Python Dependency Resolver (6-8 hours)

**Goal:** Use Python + networkx for graph algorithms

**Why Python:**
- ✅ networkx library (proven graph algorithms)
- ✅ Topological sort built-in
- ✅ Cycle detection built-in
- ✅ Fast and reliable
- ✅ Don't reimplement algorithms in Fish!

**File:** `~/dotfiles/scripts/resolve-deps.py`
````python
#!/usr/bin/env python3
"""
Faelight Config Manager - Dependency Resolver
Uses networkx for graph algorithms.
"""

import sys
import yaml
import networkx as nx
from pathlib import Path

def build_dependency_graph(dotfiles_dir: Path) -> nx.DiGraph:
    """Build directed graph of package dependencies."""
    G = nx.DiGraph()
    
    # Find all packages with metadata
    for meta_file in dotfiles_dir.glob("**/.dotfile-meta.yaml"):
        with open(meta_file) as f:
            meta = yaml.safe_load(f)
        
        pkg_name = meta['name']
        G.add_node(pkg_name, **meta)
        
        # Add dependency edges
        for dep in meta.get('depends', []):
            G.add_edge(dep, pkg_name)  # dep → pkg_name
    
    return G

def check_cycles(G: nx.DiGraph) -> list:
    """Check for circular dependencies."""
    try:
        cycles = list(nx.simple_cycles(G))
        return cycles
    except:
        return []

def resolve_dependencies(G: nx.DiGraph, package: str) -> list[str]:
    """
    Resolve dependencies for a package.
    
    Returns:
        List of packages in installation order (deps first)
    """
    if package not in G:
        raise ValueError(f"Package not found: {package}")
    
    # Get all ancestors (dependencies)
    deps = nx.ancestors(G, package)
    deps.add(package)  # Include package itself
    
    # Topological sort for install order
    subgraph = G.subgraph(deps)
    install_order = list(nx.topological_sort(subgraph))
    
    return install_order

def check_conflicts(G: nx.DiGraph, packages: list[str]) -> list[tuple]:
    """
    Check if packages conflict.
    
    Returns:
        List of (pkg1, pkg2) conflict pairs
    """
    conflicts = []
    
    for pkg in packages:
        if pkg not in G:
            continue
        
        pkg_conflicts = G.nodes[pkg].get('conflicts', [])
        
        for other in packages:
            if other in pkg_conflicts:
                conflicts.append((pkg, other))
    
    return conflicts

def main():
    if len(sys.argv) < 2:
        print("Usage: resolve-deps.py <package>", file=sys.stderr)
        sys.exit(1)
    
    package = sys.argv[1]
    dotfiles_dir = Path.home() / "dotfiles"
    
    # Build graph
    G = build_dependency_graph(dotfiles_dir)
    
    # Check for cycles
    cycles = check_cycles(G)
    if cycles:
        print(f"ERROR: Circular dependencies detected:", file=sys.stderr)
        for cycle in cycles:
            print(f"  {' → '.join(cycle)}", file=sys.stderr)
        sys.exit(1)
    
    # Resolve dependencies
    try:
        install_order = resolve_dependencies(G, package)
        
        # Output (one per line for Fish to consume)
        for pkg in install_order:
            print(pkg)
    
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
````

**Tasks:**
- [ ] Create resolve-deps.py script
- [ ] Install dependencies: `pip install networkx pyyaml`
- [ ] Test with sample metadata
- [ ] Handle edge cases (missing deps, cycles)
- [ ] Write unit tests (pytest)
- [ ] Document in FRAMEWORK.md

**Fish Integration:**
````fish
function dot-install --argument package
    echo "📦 Installing: $package"
    
    # Use Python to resolve deps
    set -l install_order (python3 ~/dotfiles/scripts/resolve-deps.py $package)
    
    if test $status -ne 0
        echo "❌ Dependency resolution failed"
        return 1
    end
    
    echo "Install order:"
    for pkg in $install_order
        echo "  - $pkg"
    end
    
    # Install each package
    for pkg in $install_order
        cd ~/dotfiles
        stow $pkg
        echo "✅ Installed: $pkg"
    end
    
    # Register active packages
    # (Add to manifest or tracking file)
end
````

**Tasks:**
- [ ] Create Fish wrapper functions
- [ ] Implement dot-install
- [ ] Implement dot-remove (check dependents)
- [ ] Implement dot-deps (show tree)
- [ ] Test complex dependency chains
- [ ] Document in FRAMEWORK.md

**Deliverables:**
- ✅ Python dependency resolver working
- ✅ Automatic installation order
- ✅ Cycle detection
- ✅ Conflict checking
- ✅ Fish integration complete

---

### Phase 4: Manifest System (6-7 hours)

**Goal:** Declarative configuration

**Manifest Format:**
````yaml
# dotfiles/manifest.yaml

profile: omarchy-laptop

active_packages:
  # Base layers
  - fish-base
  - fish-aliases
  - fish-functions
  - hypr-base
  - waybar-base
  - kitty-base
  - nvim
  - yazi
  - mako
  - gtk
  
  # Theme layer
  - fish-theme-dark
  - hypr-theme-dark
  - waybar-theme-dark
  - kitty-theme-dark
  
  # Machine-specific
  - hypr-laptop
  - waybar-laptop

environment:
  EDITOR: nvim
  BROWSER: brave
  TERMINAL: kitty

features:
  mullvad-vpn: true
  battery-management: true
  brightness-control: true

metadata:
  hostname: omarchy
  type: laptop
  last_applied: 2025-12-03T20:00:00Z
  version: 3.0.0
````

**Tasks:**
- [ ] Create manifest.yaml format spec
- [ ] Create omarchy-laptop profile
- [ ] Implement dot-apply:
````fish
  function dot-apply
      # Read manifest.yaml
      # Resolve all dependencies (via Python)
      # Check conflicts
      # Unstow conflicting packages
      # Install packages in order
      # Set environment variables
      # Update last_applied timestamp
  end
````
- [ ] Implement dot-profile (switch profiles)
- [ ] Create profile templates (laptop, desktop, server)
- [ ] Test profile switching
- [ ] Document in FRAMEWORK.md

**Deliverables:**
- ✅ Declarative manifest working
- ✅ One-command apply
- ✅ Profile system

---

### Phase 5: Integration & Testing (3-4 hours)

**Tasks:**
- [ ] Integrate with theme-switch.sh
- [ ] Integrate with theme-from-wallpaper.sh
- [ ] Add Fish aliases
- [ ] Test everything together
- [ ] Write FRAMEWORK.md
- [ ] Update COMPLETE_GUIDE.md
- [ ] Add to CHANGELOG.md

**Deliverables:**
- ✅ Fully integrated FCM
- ✅ Complete documentation
- ✅ Ready for daily use

---

**Version 3.0 Summary:**

**What You Built:**
- ⚛️ Atomic package system (composable!)
- 📋 Package metadata (intelligent!)
- 🐍 Python dependency resolver (proven algorithms!)
- 📝 Declarative manifests (state what you want!)
- 🖥️ Machine profiles (laptop/desktop/server!)

**Impact:**
Dotfiles are now **managed infrastructure** with intelligence, automation, and safety!

---

## 🏗️ Version 3.5 - Advanced Configuration Management

### Phase 1: Git Tag Snapshots (5-6 hours)

**Goal:** Use Git instead of custom YAML

**Why Git Tags:**
- ✅ Proven technology
- ✅ Smaller footprint (no duplicate YAML)
- ✅ Better tooling (git tag, git show)
- ✅ Remote backup (push tags to GitHub)

**Implementation:**
````fish
function dot-snapshot --argument description
    set -l timestamp (date +%Y-%m-%d-%H%M)
    set -l tag "faelight-$timestamp"
    
    cd ~/dotfiles
    
    # Commit current state
    git add -A
    git commit -m "Snapshot: $description" --allow-empty
    
    # Create Git tag
    git tag -a "$tag" -m "$description"
    
    # Optional: lightweight metadata file
    echo "timestamp: $(date -Iseconds)" > .dotfile-snapshots/$tag.meta
    echo "description: $description" >> .dotfile-snapshots/$tag.meta
    echo "active_packages:" >> .dotfile-snapshots/$tag.meta
    # ... add package list
    
    echo "✅ Snapshot created: $tag"
    echo "💾 Git tag: git show $tag"
end

function dot-snapshots
    echo "📸 Available Snapshots (Git Tags):"
    git tag -l "faelight-*" --sort=-creatordate | head -20
end

function dot-rollback --argument tag
    cd ~/dotfiles
    
    echo "⏪ Rolling back to: $tag"
    
    # Checkout tag
    git checkout "$tag"
    
    # Reapply configuration
    dot-apply
    
    echo "✅ Rollback complete"
end
````

**Tasks:**
- [ ] Implement snapshot functions
- [ ] Test snapshot creation
- [ ] Test rollback
- [ ] Optional: Add lightweight .meta files for quick queries
- [ ] Push tags to remote: `git push --tags`
- [ ] Document in FRAMEWORK.md

**Deliverables:**
- ✅ Git-based snapshots
- ✅ Easy rollback
- ✅ Smaller, simpler than YAML files

*(Keep rest of v3.5 as planned)*

---

## 🔥 Version 4.0 - The Phoenix Framework

*(Keep as planned - no changes)*

---

## 🎯 Complete Development Timeline

### Immediate (Next 2-4 Weeks)
- **v2.8.1-2.8.6** - Theme Intelligence Engine (28-34 hours)

### Short-term (Next 2-3 Months)
- **v2.9** - Security & Backup (6-8 hours)
- **v3.0** - FCM Foundation (20-25 hours)

### Medium-term (3-6 Months)
- **v3.5** - Advanced Management (15-20 hours)
- **v4.0** - Phoenix Framework (10-15 hours)

**Total Investment:** 80-100 hours over 6 months  
**Result:** Professional configuration management framework

---

## 🌲 Faelight Forest Principles

1. **Separation of Concerns** - Theme engine standalone
2. **Right Tool for Job** - Python for algorithms, Fish for UX
3. **Composability** - Atomic packages are MANDATORY
4. **Declarative** - State what you want (manifest.yaml)
5. **Safety** - Git tags for snapshots, validation before apply
6. **Intelligence** - Dependency resolution, conflict detection
7. **Beauty** - Not just functional, delightful
8. **Excellence** - Professional quality

---

**Current Status:** Version 2.8.0 Complete ✅  
**Next Action:** v2.8.1 - Theme Engine Foundation  
**Vision:** Infrastructure as Poetry 🌲✨

---

*Last Updated: December 03, 2025*  
*Roadmap Version: 5.0 - Architectural Refinement*
