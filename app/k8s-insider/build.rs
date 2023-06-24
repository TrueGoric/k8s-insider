use std::{
    fs::{create_dir_all, write},
    path::{Path, PathBuf},
};

use kube::CustomResourceExt;
use serde::Serialize;

const CRD_OUTPUT: &str = "../../crd";

fn main() {
    let crd_output = Path::new(CRD_OUTPUT);

    export_v1alpha1_crds(crd_output);
}

fn export_v1alpha1_crds(path: &Path) {
    use k8s_insider_core::resources::crd::v1alpha1::{
        connection::Connection, network::Network, tunnel::Tunnel,
    };

    let version_path = path.join(Path::new("v1alpha1"));

    create_dir_all(&version_path).unwrap();
    write_serialized(
        &Network::crd(),
        &get_crd_path(&version_path, Network::crd_name())
    );
    write_serialized(
        &Tunnel::crd(),
        &get_crd_path(&version_path, Tunnel::crd_name())
    );
    write_serialized(
        &Connection::crd(),
        &get_crd_path(&version_path, Connection::crd_name())
    );
}

fn write_serialized<T: Sized + Serialize>(obj: &T, path: &Path) {
    write(path, serde_yaml::to_string(obj).unwrap()).unwrap();
}

fn get_crd_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(Path::new(&format!("{name}.yaml")))
}
