

use sonos;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // find sonos devices on the network and print them out
    println!("Looking for sonos devices...");

    let devices = sonos::discover().await.unwrap();
    println!("Found {} devices", devices.len());
    
    let media_arc = devices.iter()
        .find(|d| d.model == "Sonos Arc")
        .expect("Unable to find any sonos Arc on your network");
    
    println!("{:?}", media_arc);

    let track = media_arc.track().await?;
    println!("{:?}",track);

    let result = media_arc.play().await;
    if let Ok(()) = result {
        println!("Track is playing")
    } else {
        eprintln!("Something went wrong")
    }
    Ok(())
}

