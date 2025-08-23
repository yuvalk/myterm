# MyTerm AUR Packages

This directory contains packaging files for distributing MyTerm through the Arch User Repository (AUR).

## Available Packages

### myterm
- **Package Name**: `myterm`
- **Description**: Stable release version from tagged releases
- **Source**: Release tarballs from GitHub
- **Recommended for**: Production use

### myterm-git  
- **Package Name**: `myterm-git`
- **Description**: Development version built from latest git commit
- **Source**: Git repository HEAD
- **Recommended for**: Testing latest features and development

## Installation

### Using an AUR helper (recommended)

```bash
# For stable version
yay -S myterm
# or
paru -S myterm

# For git version
yay -S myterm-git
# or  
paru -S myterm-git
```

### Manual installation

```bash
# Clone the AUR repository
git clone https://aur.archlinux.org/myterm.git
cd myterm

# Build and install
makepkg -si

# Or for git version
git clone https://aur.archlinux.org/myterm-git.git  
cd myterm-git
makepkg -si
```

## Dependencies

### Runtime Dependencies
- `wayland` - Wayland compositor support
- `libxkbcommon` - Keyboard handling
- `mesa` - OpenGL/EGL support  
- `fontconfig` - Font configuration
- `freetype2` - Font rendering
- `harfbuzz` - Text shaping

### Build Dependencies  
- `rust` - Rust compiler
- `cargo` - Rust package manager
- `wayland-protocols` - Wayland protocol definitions
- `pkgconf` - Package configuration
- `git` (git version only) - Version control

## Package Maintenance

### Updating the stable package (myterm)

1. Update `pkgver` in `PKGBUILD` to match the new release tag
2. Update `sha256sums` with the correct checksum
3. Regenerate `.SRCINFO`: `makepkg --printsrcinfo > .SRCINFO`
4. Test build: `makepkg -f`
5. Commit and push to AUR

### Updating the git package (myterm-git)

The git package automatically tracks the latest commit. Only update if:
- Dependencies change
- Build process changes
- Package metadata needs updates

Then regenerate `.SRCINFO` and test build.

## Files Description

- `PKGBUILD` - Build script for stable releases
- `PKGBUILD-git` - Build script for git version
- `.SRCINFO` - Package metadata for stable version
- `.SRCINFO-git` - Package metadata for git version
- `README.md` - This documentation

## Publishing to AUR

### Initial Setup
1. Create AUR account at https://aur.archlinux.org/
2. Upload SSH public key to your AUR account
3. Clone the empty AUR repositories:
   ```bash
   git clone ssh://aur@aur.archlinux.org/myterm.git
   git clone ssh://aur@aur.archlinux.org/myterm-git.git
   ```

### Publishing Process
1. Copy appropriate files to AUR repository directories
2. Generate `.SRCINFO`: `makepkg --printsrcinfo > .SRCINFO`  
3. Test the package: `makepkg -f`
4. Add files to git: `git add PKGBUILD .SRCINFO`
5. Commit: `git commit -m "Update to version X.X.X"`
6. Push: `git push`

## Support

For package-related issues:
- Check the [MyTerm GitHub repository](https://github.com/yuvalk/myterm)
- Report AUR packaging issues as GitHub issues
- For general AUR help, see the [Arch Wiki](https://wiki.archlinux.org/title/Arch_User_Repository)