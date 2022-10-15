%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: ytdlp-gui
Summary: a GUI for yt-dlp written in Rust
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: Apache-2.0
Group: Applications/System
Source0: %{name}-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}
for _size in 16 22 24 32 48 64 96 128 256 512 ; do \
    cp -a data/icons/$${_size}x$${_size}/ytdlp-gui.png %{buildroot}/share/icons/hicolor/$${_size}x$${_size}/apps/ytdlp-gui.png; \
done
cp -a data/applications/ytdlp-gui.desktop %{buildroot}/share/applications/ytdlp-gui.desktop


%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
%{_datadir}/applications/%{name}.desktop
%{_datadir}/icons/hicolor/*/apps/ytdlp-gui.png
