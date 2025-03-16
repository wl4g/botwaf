use crate::{
    config::config::{GIT_BUILD_DATE, GIT_COMMIT_HASH, GIT_VERSION},
    logging,
    verifier::verifier_handler::VerifierHandlerFactory,
};

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // http://www.network-science.de/ascii/#larry3d,graffiti,basic,drpepper,rounded,roman
    let ascii_name = r#"
    ____            __    __      __               ___      __  __                        ___                     
    /\  _`\         /\ \__/\ \  __/\ \            /'___\    /\ \/\ \                 __  /'___\ __                 
    \ \ \L\ \    ___\ \ ,_\ \ \/\ \ \ \     __   /\ \__/    \ \ \ \ \     __   _ __ /\_\/\ \__//\_\     __   _ __  
     \ \  _ <'  / __`\ \ \/\ \ \ \ \ \ \  /'__`\ \ \ ,__\    \ \ \ \ \  /'__`\/\`'__\/\ \ \ ,__\/\ \  /'__`\/\`'__\
      \ \ \L\ \/\ \L\ \ \ \_\ \ \_/ \_\ \/\ \L\.\_\ \ \_/     \ \ \_/ \/\  __/\ \ \/ \ \ \ \ \_/\ \ \/\  __/\ \ \/ 
       \ \____/\ \____/\ \__\\ `\___x___/\ \__/.\_\\ \_\       \ `\___/\ \____\\ \_\  \ \_\ \_\  \ \_\ \____\\ \_\ 
        \/___/  \/___/  \/__/ '\/__//__/  \/__/\/_/ \/_/        `\/__/  \/____/ \/_/   \/_/\/_/   \/_/\/____/ \/_/ 
                                                                                                                   
"#;
    eprintln!("");
    eprintln!("{}", ascii_name);
    eprintln!("                Program Version: {:?}", GIT_VERSION);
    eprintln!("                Package Version: {:?}", env!("CARGO_PKG_VERSION").to_string());
    eprintln!("                Git Commit Hash: {:?}", GIT_COMMIT_HASH);
    eprintln!("                 Git Build Date: {:?}", GIT_BUILD_DATE);

    logging::init_components().await;

    VerifierHandlerFactory::start().await;

    Ok(())
}
