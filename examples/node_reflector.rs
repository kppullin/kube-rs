#[macro_use] extern crate log;
use kube::{
    api::{Api, Reflector},
    client::APIClient,
    config,
};

fn main() -> Result<(), failure::Error> {
    std::env::set_var("RUST_LOG", "info,kube=trace");
    env_logger::init();
    let config = config::load_kube_config().expect("failed to load kubeconfig");
    let client = APIClient::new(config);

    let resource = Api::v1Node(client);
    let rf = Reflector::new(resource)
        .labels("role=master")
        .init()?;

    // rf is initialized with full state, which can be extracted on demand.
    // Output is Map of name -> Node
    rf.read()?.into_iter().for_each(|object| {
        info!("Found node {} ({:?}) running {:?} with labels: {:?}",
            object.metadata.name,
            object.spec.provider_id.unwrap(),
            object.status.unwrap().conditions.unwrap(),
            object.metadata.labels,
        );
    });

    // r needs to have `r.poll()?` called continuosly to keep state up to date:
    loop {
        rf.poll()?;
        let deploys = rf.read()?.into_iter().map(|object| object.metadata.name).collect::<Vec<_>>();
        info!("Current nodes: {:?}", deploys);
    }
}
