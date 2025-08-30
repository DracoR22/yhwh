pub async fn load_file_string_from_env(directory: &str, file_name: &str) -> anyhow::Result<String> {
    let txt = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res")
            .join(directory)
            .join(file_name);
        std::fs::read_to_string(path)?
    };

    Ok(txt)
}

pub async fn load_file_binary_from_env(directory: &str, file_name: &str) -> anyhow::Result<Vec<u8>> {
    #[cfg(not(target_arch = "wasm32"))]
    let data = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res")
            .join(directory)
            .join(file_name);
        std::fs::read(path)?
    };

    Ok(data)
}

// pub fn load_file_string_from_dir(directory: &str) {
//     let file = std::fs::read("").unwrap();
// }