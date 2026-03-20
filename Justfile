#Runs backend
run-backend:
    cd backend && cargo run

#Runs frontend
run-frontend:
    cd frontend && bun install
    cd frontend && bun run tauri dev

#Initializes frontend and runs it
initialize-frontend:
    cd frontend && bun install && bun run tauri dev

CA_KEY := "ca.key"
CA_CRT := "ca.crt"
SERVER_DIR := "backend"
CLIENT_DIR := "frontend/src-tauri"

# Setups the keys localy to be used by client and server
setup-keys:
    @echo "--- Gerando Infraestrutura de Chaves (mTLS) ---"
    
    # 1. Gerar Root CA (se não existir)
    openssl req -x509 -sha256 -nodes -days 365 -newkey rsa:2048 \
        -keyout {{CA_KEY}} -out {{CA_CRT}} \
        -subj "/CN=Verita Root CA"
    # 2. Configurar Servidor (Backend)
    @echo "Configurando Servidor..."
    openssl req -newkey rsa:2048 -nodes -keyout {{SERVER_DIR}}/server.key \
        -out {{SERVER_DIR}}/server.csr -subj "/CN=localhost"
    
    echo "subjectAltName=DNS:localhost,IP:127.0.0.1" > {{SERVER_DIR}}/server_ext.cnf
    
    openssl x509 -req -extfile {{SERVER_DIR}}/server_ext.cnf -days 365 \
        -in {{SERVER_DIR}}/server.csr -CA {{CA_CRT}} -CAkey {{CA_KEY}} \
        -CAcreateserial -out {{SERVER_DIR}}/server.crt
    
    openssl pkcs8 -topk8 -nocrypt -in {{SERVER_DIR}}/server.key -out {{SERVER_DIR}}/server_pkcs8.pem
    # 3. Configurar Cliente (Frontend/Tauri)
    @echo "Configurando Cliente..."
    openssl req -newkey rsa:2048 -nodes -keyout {{CLIENT_DIR}}/client.key \
        -out {{CLIENT_DIR}}/client.csr -subj "/CN=verita-client"
    
    openssl x509 -req -days 365 -in {{CLIENT_DIR}}/client.csr \
        -CA {{CA_CRT}} -CAkey {{CA_KEY}} -CAcreateserial \
        -out {{CLIENT_DIR}}/client.crt
    openssl pkcs8 -topk8 -nocrypt -in {{CLIENT_DIR}}/client.key -out {{CLIENT_DIR}}/client_pkcs8.pem

    @echo "Criando {{SERVER_DIR}}/.env..."
    printf 'QUIC_KEY_PATH=server_pkcs8.pem\nQUIC_CERT_PATH=server.crt\nQUIC_CA_PATH=../ca.crt\nSERVER_PORT=4433\n' > {{SERVER_DIR}}/.env
    @echo "Criando {{CLIENT_DIR}}/.env..."
    printf 'QUIC_CA_CERT_PATH=../../ca.crt\nQUIC_CERT_PATH=client.crt\nQUIC_KEY_PATH=client_pkcs8.pem\nSERVER_PORT=4433\n' > {{CLIENT_DIR}}/.env
    @echo "--- Setup Concluído com Sucesso! ---"
    @echo "Arquivos PKCS8 e .env prontos para o Rustls."

    

# Clears the keys of the project
clean-keys:
    rm -f *.crt *.key *.srl
    rm -f {{SERVER_DIR}}/*.crt {{SERVER_DIR}}/*.key {{SERVER_DIR}}/*.csr {{SERVER_DIR}}/*.cnf {{SERVER_DIR}}/*.pem
    rm -f {{CLIENT_DIR}}/*.crt {{CLIENT_DIR}}/*.key {{CLIENT_DIR}}/*.csr {{CLIENT_DIR}}/*.pem
