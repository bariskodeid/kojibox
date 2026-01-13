<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kojibox - It Works!</title>
    <style>
        body { font-family: -apple-system, system-ui, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif; background: #f1f0ea; color: #333; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; }
        .card { background: white; padding: 2rem; border: 2px solid #333; box-shadow: 4px 4px 0 #333; max-width: 480px; width: 100%; }
        h1 { margin-top: 0; }
        .status { margin-top: 1rem; padding: 1rem; background: #e8f4e8; border: 1px solid #6fb56f; font-size: 0.9rem; }
    </style>
</head>
<body>
    <div class="card">
        <h1>It Works!</h1>
        <p>Your Kojibox environment is running.</p>
        <div class="status">
            <strong>PHP Version:</strong> <?php echo phpversion(); ?><br>
            <strong>Server Software:</strong> <?php echo $_SERVER['SERVER_SOFTWARE']; ?>
        </div>
        <p><small>Place your projects in separate folders and map them using the Kojibox Dashboard.</small></p>
    </div>
</body>
</html>
