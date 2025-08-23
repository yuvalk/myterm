#!/bin/bash
# AUR package maintenance script for MyTerm

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to update PKGBUILD version
update_pkgbuild_version() {
    local pkgbuild_file="$1"
    local new_version="$2"
    
    log_info "Updating $pkgbuild_file to version $new_version"
    
    sed -i "s/^pkgver=.*/pkgver=$new_version/" "$pkgbuild_file"
    sed -i "s/^pkgrel=.*/pkgrel=1/" "$pkgbuild_file"
    
    log_success "Updated $pkgbuild_file"
}

# Function to generate checksums
generate_checksums() {
    local pkgbuild_file="$1"
    
    log_info "Generating checksums for $pkgbuild_file"
    
    cd "$(dirname "$pkgbuild_file")"
    
    # Download source and generate checksums
    makepkg -g >> /tmp/checksums.tmp
    
    # Replace SKIP with actual checksums
    local checksums=$(tail -1 /tmp/checksums.tmp)
    sed -i "s/^sha256sums=.*/$(echo "$checksums")/" "$(basename "$pkgbuild_file")"
    
    rm -f /tmp/checksums.tmp
    
    log_success "Updated checksums for $pkgbuild_file"
}

# Function to generate .SRCINFO
generate_srcinfo() {
    local pkgbuild_file="$1"
    local srcinfo_file="${pkgbuild_file%PKGBUILD}.SRCINFO"
    
    log_info "Generating $srcinfo_file"
    
    cd "$(dirname "$pkgbuild_file")"
    makepkg --printsrcinfo > "$(basename "$srcinfo_file")"
    
    log_success "Generated $srcinfo_file"
}

# Function to test build
test_build() {
    local pkgbuild_file="$1"
    
    log_info "Testing build for $pkgbuild_file"
    
    cd "$(dirname "$pkgbuild_file")"
    
    # Clean previous builds
    rm -rf src/ pkg/ *.pkg.tar.zst
    
    # Test build
    if makepkg -f --noconfirm; then
        log_success "Build test passed for $pkgbuild_file"
        # Clean up test build
        rm -rf src/ pkg/ *.pkg.tar.zst
        return 0
    else
        log_error "Build test failed for $pkgbuild_file"
        return 1
    fi
}

# Main function
main() {
    local action="${1:-help}"
    
    case "$action" in
        "update-stable")
            if [[ $# -ne 2 ]]; then
                log_error "Usage: $0 update-stable <version>"
                exit 1
            fi
            
            local version="$2"
            local pkgbuild="$SCRIPT_DIR/PKGBUILD"
            
            update_pkgbuild_version "$pkgbuild" "$version"
            generate_checksums "$pkgbuild"
            generate_srcinfo "$pkgbuild"
            test_build "$pkgbuild"
            
            log_success "Stable package updated to version $version"
            log_info "Next steps:"
            log_info "1. Review changes: git diff"
            log_info "2. Commit: git add PKGBUILD .SRCINFO && git commit -m 'Update to $version'"
            log_info "3. Push to AUR: git push"
            ;;
            
        "update-git")
            local pkgbuild="$SCRIPT_DIR/PKGBUILD-git"
            local srcinfo="$SCRIPT_DIR/.SRCINFO-git"
            
            generate_srcinfo "$pkgbuild"
            test_build "$pkgbuild"
            
            log_success "Git package updated"
            log_info "Next steps:"
            log_info "1. Review changes: git diff"
            log_info "2. Commit: git add PKGBUILD .SRCINFO && git commit -m 'Update git package'"
            log_info "3. Push to AUR: git push"
            ;;
            
        "test")
            local package="${2:-both}"
            
            case "$package" in
                "stable")
                    test_build "$SCRIPT_DIR/PKGBUILD"
                    ;;
                "git")
                    test_build "$SCRIPT_DIR/PKGBUILD-git"
                    ;;
                "both")
                    test_build "$SCRIPT_DIR/PKGBUILD"
                    test_build "$SCRIPT_DIR/PKGBUILD-git"
                    ;;
                *)
                    log_error "Unknown package: $package"
                    log_info "Available packages: stable, git, both"
                    exit 1
                    ;;
            esac
            ;;
            
        "help"|*)
            echo "MyTerm AUR Package Maintenance Script"
            echo
            echo "Usage: $0 <action> [options]"
            echo
            echo "Actions:"
            echo "  update-stable <version>  Update stable package to specific version"
            echo "  update-git               Update git package metadata"  
            echo "  test [stable|git|both]   Test build packages (default: both)"
            echo "  help                     Show this help message"
            echo
            echo "Examples:"
            echo "  $0 update-stable 0.2.0"
            echo "  $0 update-git"
            echo "  $0 test stable"
            ;;
    esac
}

# Check requirements
if ! command -v makepkg &> /dev/null; then
    log_error "makepkg not found. Please install base-devel package."
    exit 1
fi

# Run main function with all arguments
main "$@"