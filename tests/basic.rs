#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::time::timeout;
    use reqwest::Client;

    #[tokio::test]
    async fn test_server_starts() {
        // Запустить сервер в отдельном процессе
        let child = std::process::Command::new("cargo")
            .args(&["run", "--release"])
            .spawn()
            .expect("Failed to start server");

        // Дать серверу время на запуск
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Проверить, что сервер отвечает
        let client = Client::new();
        let result = timeout(
            Duration::from_secs(5),
            client.get("http://localhost:3000").send()
        ).await;

        // Проверить результат
        match result {
            Ok(response_result) => {
                match response_result {
                    Ok(response) => {
                        assert!(response.status().is_success());
                    },
                    Err(e) => {
                        panic!("Failed to connect to server: {}", e);
                    }
                }
            },
            Err(_) => {
                panic!("Request to server timed out");
            }
        }

        // Завершить процесс сервера
        child.kill().expect("Failed to kill server process");
    }
}