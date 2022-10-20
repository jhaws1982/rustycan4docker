# VXCAN Network Plugin for Docker

This Docker plugin provides the ability to create VXCAN tunnels for Docker containers. It is based heavily on the work by Christain Gagneraud (https://gitlab.com/chgans/can4docker) and Wiktor S. Ovalle Correa (https://github.com/wsovalle/docker-vxcan).

This plugin has essentially taken the Python implementation and rewrote it in Rust as a way for me to learn Rust and to speed up the plugin startup time, which on an embedded system was way too long when written in Python.

## Requirements

Requires that the vxcan and can-gw modules are built-in or loaded into the kernel.
```
sudo modprobe vxcan
sudo modprobe can-gw
```

## Available Options
**vxcan.id**: Numerical identifier of the interface (i.e., 0 for can0, or 1 for can1). Default is 0.

**vxcan.dev**: Specify the CAN device to use on the host. If the device is present (i.e., a physical CAN device) then it will be used as is; otherwise, a virtual CAN interface is created to use. Default is 'vcan'.

**vxcan.peer**: Prefix for the peer device (i.e., endpoint) to use in the container. This is combined with the vxcan.id to produce an interface name (e.g., vxcanp0). Default is 'vcanp'.

## Usage

### Docker
```
# Create a couple Docker containers to test in separate terminals
docker run --rm -it --name a1 alpine
docker run --rm -it --name a2 alpine

# Create the network
docker network create --driver rustyvxcan -o vxcan.dev=vcan -o vxcan.id=0 -o vxcan.peer=vxcanp rust_can1

# Connect the network to the containers
docker network connect rust_can1 a1
docker network connect rust_can1 a2

# Check that the cangw rules are present (twelve total)
cangw -L

# Check that the required interfaces are present (one vcan0, 2 vxcanXXXXXXXX)
ip link

# In the container terminals
apk add can-utils
cangen vxcanp0 # from one container
candump vxcanp0 # from the other container
cangen vcan0 # from the host

# Remove the network (after closing the containers)
docker network rm rust_can1
```

### Compose Application
docker-compose applications can make use of the plugin as well.
```
networks:
  canbus:
    driver: rustyvxcan
    driver_opts:
      vxcan.dev: can
      vxcan.peer: can
      vxcan.id: 0
```

### Plugin Installation
This is typically just used as a simple systemd service, rather than being installed with `docker plugin install <name>`.