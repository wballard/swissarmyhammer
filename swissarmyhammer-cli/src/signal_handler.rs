use tokio::signal;
use tracing::info;

pub async fn setup_signal_handlers() -> anyhow::Result<()> {
    tokio::spawn(async {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                info!("Received Ctrl+C signal, shutting down gracefully...");
            },
            _ = terminate => {
                info!("Received terminate signal, shutting down gracefully...");
            },
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signal_handler_setup() {
        // Simply test that the function can be called without panicking
        let result = setup_signal_handlers().await;
        assert!(result.is_ok());
    }
}
