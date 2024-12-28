use super::ShutdownError;
use super::ShutdownManager;

#[tokio::test]
#[should_panic(expected = "at least one exit condition")]
async fn builder_panics_without_conditions() {
    ShutdownManager::<()>::builder().build();
}

#[tokio::test]
async fn graceful_shutdown_timeout() {
    // The first task waits a long time then sets a flag.
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag_setter = std::sync::Arc::clone(&flag);
    let task_long = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5 * 60)).await;
        flag_setter.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    });

    // The second task exists immediately to trigger shutdown logic.
    let task_shutdown = tokio::spawn(async { Ok(()) });

    // Wait for shutdown.
    let mut shutdown = ShutdownManager::builder();
    shutdown
        .graceful_shutdown_timeout(std::time::Duration::from_millis(10))
        .watch_tokio(task_long)
        .watch_tokio(task_shutdown);
    let shutdown = shutdown.build();
    let start_time = std::time::Instant::now();
    let _ = shutdown.wait().await;
    let test_duration = start_time.elapsed();

    // Ensure the flag is not set (task was cancelled) and the test was short.
    let flag = flag.load(std::sync::atomic::Ordering::SeqCst);
    assert!(!flag);
    assert!(test_duration.as_millis() < 200);
}

#[tokio::test]
async fn shutdown_handles() {
    let mut shutdown = ShutdownManager::builder();

    // Task to wait for exit and send a signal back.
    let exit = shutdown.shutdown_handle();
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag_setter = std::sync::Arc::clone(&flag);
    let task_exit = tokio::spawn(async move {
        exit.wait().await;
        flag_setter.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    });

    // Task to exit immediately (and therefore start shutdown).
    let task_shutdown = tokio::spawn(async { Ok(()) });

    // Wait for shutdown.
    shutdown.watch_tokio(task_exit).watch_tokio(task_shutdown);
    let shutdown = shutdown.build();

    let test_timeout = std::time::Duration::from_secs(5);
    tokio::select! {
        result = shutdown.wait() => assert!(result.is_ok()),
        _ = tokio::time::sleep(test_timeout) => panic!("ShutdownManager blocked too long"),
    };

    let flag = flag.load(std::sync::atomic::Ordering::SeqCst);
    assert!(flag);
}

#[tokio::test]
async fn shutdown_notifications() {
    let mut shutdown = ShutdownManager::builder();

    // Task to wait for exit and send a signal back.
    let exit = shutdown.shutdown_notification();
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag_setter = std::sync::Arc::clone(&flag);
    let task_exit = tokio::spawn(async move {
        exit.await;
        flag_setter.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    });

    // Task to exit immediately (and therefore start shutdown).
    let task_shutdown = tokio::spawn(async { Ok(()) });

    // Wait for shutdown.
    shutdown.watch_tokio(task_exit).watch_tokio(task_shutdown);
    let shutdown = shutdown.build();

    let test_timeout = std::time::Duration::from_secs(5);
    tokio::select! {
        result = shutdown.wait() => assert!(result.is_ok()),
        _ = tokio::time::sleep(test_timeout) => panic!("ShutdownManager blocked too long"),
    };

    let flag = flag.load(std::sync::atomic::Ordering::SeqCst);
    assert!(flag);
}

#[tokio::test]
async fn wait_for_task() {
    let task = tokio::spawn(async { Ok("test result") });
    let shutdown = {
        let mut shutdown = ShutdownManager::builder();
        shutdown.watch_tokio(task);
        shutdown.build()
    };
    let result = shutdown.wait().await.unwrap();
    assert_eq!(result, "test result");
}

#[tokio::test]
async fn wait_for_task_many() {
    let task_one = tokio::spawn(async {
        let delay = std::time::Duration::from_millis(100);
        tokio::time::sleep(delay).await;
        Ok("test result")
    });
    let task_two = tokio::spawn(async { anyhow::bail!("test error") });
    let mut shutdown = ShutdownManager::builder();
    shutdown.watch_tokio(task_one).watch_tokio(task_two);
    let shutdown = shutdown.build();
    let result = shutdown.wait().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn wait_for_task_panic() {
    let task: tokio::task::JoinHandle<anyhow::Result<()>> =
        tokio::spawn(async { panic!("this task panics") });
    let shutdown = {
        let mut shutdown = ShutdownManager::builder();
        shutdown.watch_tokio(task);
        shutdown.build()
    };
    let result = shutdown.wait().await;
    match result {
        Ok(_) => panic!("expected task to report an error"),
        Err(error) if error.is::<ShutdownError>() => {
            let error: ShutdownError = error.downcast().unwrap();
            match error {
                ShutdownError::TokioTaskPanic => (),
                error => panic!("expected task to panic but got {:?}", error),
            }
        }
        Err(error) => panic!("expected task to panic but got {:?}", error),
    }
}
