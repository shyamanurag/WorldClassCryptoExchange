 for pair in config.trading_pairs.clone() {
                info!("Adding trading pair: {}", pair);
                manager.add_symbol(pair).await?;
            }
            
            // Initialize wallet system
            let wallet_handle = tokio::spawn(async move {
                if let Err(e) = wallet::run(config.clone(), false).await {
                    error!("Wallet system error: {}", e);
                }
            });
            
            // Initialize API service
            let mut api_service = api::ApiService::new(
                Arc::clone(&manager),
                user_repo,
                asset_repo,
                trading_pair_repo,
                order_repo,
                trade_repo,
                wallet_repo,
                account_repo,
                deposit_repo,
                withdrawal_repo,
                Arc::clone(&auth_service),
            );
            
            // Start API service
            api_service.start("0.0.0.0", port).await?;
            
            info!("All components started successfully");
            
            // Wait for shutdown signal
            wait_for_shutdown().await?;
            
            // Shutdown API service
            api_service.stop().await?;
            
            // Wait for wallet system to shut down
            if let Err(e) = wallet_handle.await {
                error!("Error waiting for wallet system to shut down: {}", e);
            }
        },
    }
    
    info!("WorldClass Crypto Exchange shut down gracefully");
    
    Ok(())
}

/// Wait for shutdown signal (Ctrl+C)
async fn wait_for_shutdown() -> Result<()> {
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, starting graceful shutdown");
            Ok(())
        }
        Err(e) => {
            error!("Failed to listen for shutdown signal: {}", e);
            Err(anyhow::anyhow!("Failed to listen for shutdown signal"))
        }
    }
}
