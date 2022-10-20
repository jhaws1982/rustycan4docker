/*
 * Filename: endpoint.rs
 * Created Date: Tuesday, October 18th 2022, 5:15:15 pm
 * Author: Jonathan Haws
 *
 * Copyright (c) 2022 WiTricity
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use truncrate::*;

#[derive(Clone)]
pub struct Endpoint {
    pub uid: String,
    pub device: String,
    pub peer: String,
    created: bool,
}

impl Endpoint {
    pub fn new(uid: String) -> Self {
        println!("Creating a new endpoint: {uid}");
        let ifcs = interfaces::Interface::get_all().unwrap();

        let mut exists: bool = false;
        let newifc = format!("vxcan{}", uid.truncate_to_byte_offset(8));
        let peerifc = format!("{newifc}p");

        for i in ifcs.into_iter() {
            if i.name.eq(&newifc) {
                exists = true;
            }
        }

        if !exists {
            std::process::Command::new("ip")
                .arg("link")
                .arg("add")
                .arg("dev")
                .arg(&newifc)
                .arg("type")
                .arg("vxcan")
                .arg("peer")
                .arg("name")
                .arg(&peerifc)
                .output()
                .expect("failed to add VXCAN device");
            std::process::Command::new("ip")
                .arg("link")
                .arg("set")
                .arg("up")
                .arg(&newifc)
                .output()
                .expect("failed to start VXCAN device");
        }
        println!(
            "Creating VXCAN tunnel with settings: device='{}', peer='{}'",
            newifc, peerifc
        );
        Endpoint {
            uid: uid,
            device: newifc,
            peer: peerifc,
            created: !exists,
        }
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        if self.created {
            // Actually delete the network interface
            std::process::Command::new("ip")
                .arg("link")
                .arg("set")
                .arg("down")
                .arg(&self.device)
                .output()
                .expect("failed to start VCAN device");
            std::process::Command::new("ip")
                .arg("link")
                .arg("del")
                .arg("dev")
                .arg(&self.device)
                .arg("type")
                .arg("vxcan")
                .output()
                .expect("failed to remove VCAN device");

            println!(
                "Dropping Endpoint object with {}, {}",
                self.device, self.peer,
            );
        }
    }
}
