inherit systemd

RDEPENDS:${PN} += "\
    can-utils \
    can-utils-isotp \
    docker \
"

do_install:append() {
    install -d ${D}${systemd_system_unitdir}
    install -m 0644 ${S}/rustyvxcan.service ${D}${systemd_system_unitdir}/

    install -d ${D}${bindir}
    install -m 0755 ${WORKDIR}/build/target/arm-lmp-linux-gnueabi/release/rustycan4docker ${D}${bindir}/
}

SYSTEMD_AUTO_ENABLE:${PN} = "enable"
SYSTEMD_SERVICE:${PN} = "rustyvxcan.service"
FILES:${PN} += " \
    ${systemd_system_unitdir}/rustyvxcan.service \
    ${bindir}/rustycan4docker \
"
