#Runs backend
run-backend:
    cargo run -p backend

#Runs frontend
run-frontend:
    cd frontend && bun run tauri dev

#Initializes frontend and runs it
initialize-frontend:
    cd frontend && bun install && bun run tauri dev
