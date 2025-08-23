Name:           myterm
Version:        0.1.0
Release:        1%{?dist}
Summary:        Modern terminal emulator for Sway and Wayland

License:        MIT or Apache-2.0
URL:            https://github.com/yuvalk/myterm
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo
BuildRequires:  wayland-devel
BuildRequires:  libxkbcommon-devel
BuildRequires:  mesa-libEGL-devel
BuildRequires:  fontconfig-devel
BuildRequires:  freetype-devel
BuildRequires:  harfbuzz-devel
BuildRequires:  pkgconfig

Requires:       wayland
Requires:       libxkbcommon
Requires:       mesa-libEGL
Requires:       fontconfig
Requires:       freetype
Requires:       harfbuzz

%description
MyTerm is a fast, modern terminal emulator designed specifically for
Sway window manager and Wayland protocol. It provides excellent
performance, Unicode support, and seamless integration with Sway's
tiling capabilities.

Features include:
- Native Wayland support
- True color (24-bit) support
- GPU acceleration
- Configurable fonts and themes
- ANSI/VT100/VT220 terminal emulation
- Scrollback buffer with search
- Copy/paste integration
- HiDPI support

%prep
%setup -q

%build
cargo build --release

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications
install -m 755 target/release/myterm %{buildroot}%{_bindir}/myterm
install -m 644 myterm.desktop %{buildroot}%{_datadir}/applications/myterm.desktop

%files
%license LICENSE-MIT LICENSE-APACHE
%doc README.md
%{_bindir}/myterm
%{_datadir}/applications/myterm.desktop

%changelog
* Wed Aug 23 2025 Yuval K <yuvalk@example.com> - 0.1.0-1
- Initial package