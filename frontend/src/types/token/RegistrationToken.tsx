export interface RegistrationTokenClaims {
  user_id: number;
  username: string;
  email: string;
  exp: number;
};