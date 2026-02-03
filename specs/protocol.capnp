@0xe6c3191109badb31

struct UserId {
  id @0 :UInt64;
}

struct LoginRequest {
  userId @0 :UInt64;
  passwordHashed @1 :Data; #Argon2(password + 16bytes salt)
}

struct RegisterRequest {
  username @0 :Text;
  passwordHashed @1 :Data; #Argon2(password + 16bytes salt)
}

struct RegisterRespose {
  userId @0 UserId;
}
