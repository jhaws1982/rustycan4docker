/*
 * Filename: network.rs
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

use crate::endpoint::Endpoint;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Error;
use std::sync::Arc;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JoinResponse {
    SrcName: String,
    DstPrefix: String,
}

pub struct Network {
    device: String,
    peer: String,
    canid: u32,
    ifc: String,
    created: bool,
    endpoint_list: Arc<RwLock<HashMap<String, Endpoint>>>,
    rules_list: Arc<RwLock<Vec<(String, String)>>>,
}

impl Network {
    pub fn new(device: String, peer: String, canid: u32) -> Self {
        let ifcs = interfaces::Interface::get_all().unwrap();

        let mut exists: bool = false;
        let newifc = format!("{device}{canid}");

        for i in ifcs.into_iter() {
            if i.name.eq(&newifc) {
                exists = true;
            }
        }

        if !exists {
            println!(" -> Creating interface {newifc}...");
            std::process::Command::new("ip")
                .arg("link")
                .arg("add")
                .arg("dev")
                .arg(&newifc)
                .arg("type")
                .arg("vcan")
                .output()
                .expect(" !! Failed to add VCAN device");
            std::process::Command::new("ip")
                .arg("link")
                .arg("set")
                .arg("up")
                .arg(&newifc)
                .output()
                .expect(" !! Failed to start VCAN device");
        }
        println!(
            " -> Creating network with settings: device='{}', peer='{}', id='{}' -- new device? {}",
            device,
            peer,
            canid.to_string(),
            !exists
        );
        Network {
            device: device,
            peer: peer,
            canid: canid,
            ifc: newifc,
            created: !exists,
            endpoint_list: Arc::new(RwLock::new(HashMap::new())),
            rules_list: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn endpoint_add(&mut self, ep: Endpoint) {
        // Add the endpoint to the list
        self.endpoint_list.write().insert(ep.uid.clone(), ep);
    }

    pub fn endpoint_remove(&mut self, uid: String) {
        let mut map = self.endpoint_list.write();
        if map.contains_key(&uid) {
            println!(" -> Endpoint {uid} exists...removing!");
            map.remove(&uid);
        }
    }

    pub fn endpoint_attach(
        &mut self,
        epuid: String,
        _namespace: String,
        peer: String,
    ) -> Result<JoinResponse, Error> {
        let map = self.endpoint_list.read();
        let ep = map.get(&epuid).unwrap();

        // Add cangw rules: self->endpoint, endpoint->self
        self.add_cangw_rule(&self.ifc, &ep.device);
        self.add_cangw_rule(&ep.device, &self.ifc);

        for (uid, endpt) in map.iter() {
            if uid.ne(&epuid) {
                // Add cangw rules: other->endpoint, endpoint->other
                self.add_cangw_rule(&endpt.device, &ep.device);
                self.add_cangw_rule(&ep.device, &endpt.device);
            }
        }

        let mut peerifc = &peer;
        if peer.is_empty() {
            peerifc = &self.peer;
        }

        let rsp = JoinResponse {
            SrcName: ep.peer.clone(),
            DstPrefix: (*peerifc).clone(),
        };
        Ok(rsp)
    }

    pub fn endpoint_detach(&mut self, epuid: String) {
        let map = self.endpoint_list.read();
        let ep = map.get(&epuid).unwrap();

        for (uid, endpt) in map.iter() {
            if uid.ne(&epuid) {
                // Remove cangw rules: other->endpoint, endpoint->other
                self.remove_cangw_rule(&endpt.device, &ep.device);
                self.remove_cangw_rule(&ep.device, &endpt.device);
            }
        }

        // Remove cangw rules: self->endpoint, endpoint->self
        self.remove_cangw_rule(&ep.device, &self.ifc);
        self.remove_cangw_rule(&self.ifc, &ep.device);
    }

    fn add_cangw_rule(&self, src: &String, dst: &String) {
        println!(" -> Adding cangw rule for {src} to {dst}");

        std::process::Command::new("cangw")
            .arg("-A")
            .arg("-s")
            .arg(&src)
            .arg("-d")
            .arg(&dst)
            .arg("-e")
            .output()
            .expect(" !! Failed to add cangw rule");

        std::process::Command::new("cangw")
            .arg("-A")
            .arg("-s")
            .arg(&src)
            .arg("-d")
            .arg(&dst)
            .arg("-eX")
            .output()
            .expect(" !! Failed to add cangw extended rule");

        self.rules_list.write().push((src.clone(), dst.clone()));
    }

    fn remove_cangw_rule(&self, src: &String, dst: &String) {
        let mut rules = self.rules_list.write();
        if rules.contains(&(src.clone(), dst.clone())) {
            println!(" -> Removing cangw rule for {src} to {dst}");

            std::process::Command::new("cangw")
                .arg("-D")
                .arg("-s")
                .arg(&src)
                .arg("-d")
                .arg(&dst)
                .arg("-e")
                .output()
                .expect(" !! Failed to remove cangw rule");

            std::process::Command::new("cangw")
                .arg("-D")
                .arg("-s")
                .arg(&src)
                .arg("-d")
                .arg(&dst)
                .arg("-eX")
                .output()
                .expect(" !! Failed to remove cangw extended rule");

            let index = rules
                .iter()
                .position(|x| *x == (src.clone(), dst.clone()))
                .unwrap();
            rules.remove(index);
        }
    }
}

impl Drop for Network {
    fn drop(&mut self) {
        if self.created {
            let ifc = format!("{}{}", self.device, self.canid);

            // Actually delete the network interface
            std::process::Command::new("ip")
                .arg("link")
                .arg("set")
                .arg("down")
                .arg(&ifc)
                .output()
                .expect(" !! Failed to stop VCAN device");
            std::process::Command::new("ip")
                .arg("link")
                .arg("del")
                .arg("dev")
                .arg(&ifc)
                .arg("type")
                .arg("vcan")
                .output()
                .expect(" !! Failed to remove VCAN device");

            println!(
                " -> Dropping network object: device={}, peer={}, id={}",
                self.device,
                self.peer,
                self.canid.to_string()
            );
        }
    }
}
