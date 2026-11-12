export type Permission = "view_admin" | "manage_users" | "manage_roles";

export type UserStatus = "pending" | "active" | "suspended";

export type Role = {
  id: string;
  name: string;
  label: string;
  isSystem: boolean;
};

export type RoleOption = {
  name: string;
  label: string;
};

export type SessionUser = {
  id: string;
  name: string;
  email: string;
  emailVerified: boolean;
  image: string | null;
  status: UserStatus;
  phoneNumber: string | null;
  jobTitle: string | null;
  department: string | null;
  twoFactorEnabled: boolean;
  role: Role | null;
  permissions: Permission[];
};

export type Session = {
  id: string;
  createdAt: string;
  updatedAt: string;
  expiresAt: string;
  ipAddress: string | null;
  userAgent: string | null;
};

export type SessionData = {
  session: Session;
  user: SessionUser;
  passwordConfirmed: boolean;
  canDeleteAccount: boolean;
  superadminCount: number;
};

export type SessionEnvelope = {
  session: SessionData | null;
};

export type SignUpUser = {
  id: string;
  name: string;
  email: string;
  status: UserStatus;
};

export type SignUpResponse = {
  user: SignUpUser;
};

export type SignInResponse =
  { user: SessionUser } | { twoFactorRedirect: true };

export type UserRow = {
  id: string;
  name: string;
  email: string;
  status: UserStatus;
  role: Role | null;
  createdAt: string;
  canManage: boolean;
  canDelete: boolean;
};

export type UserListResponse = {
  users: UserRow[];
  total: number;
  page: number;
  pageCount: number;
};

export type UserDetail = {
  id: string;
  name: string;
  email: string;
  phoneNumber: string | null;
  jobTitle: string | null;
  department: string | null;
  role: string | null;
  status: UserStatus;
};

export type EditableUser = {
  user: UserDetail;
  roles: RoleOption[];
};

export type UserMeta = {
  roles: RoleOption[];
  registrationApproval: boolean;
  superadminCount: number;
};

export type PermissionOption = {
  id: string;
  key: string;
  label: string;
  description: string | null;
};

export type RoleWithPermissions = {
  id: string;
  name: string;
  label: string;
  description: string | null;
  isSystem: boolean;
  permissions: string[];
  userCount: number;
};

export type RolesView = {
  roles: RoleWithPermissions[];
  permissions: PermissionOption[];
};

export type DashboardData = {
  totalUsers: number;
  activeUsers: number;
  pendingUsers: number;
  rolesCount: number;
  roleData: { name: string; label: string; users: number }[];
  signupData: { date: string; users: number }[];
  recentUsers: {
    id: string;
    name: string;
    email: string;
    status: UserStatus;
    createdAt: string;
    role: Role | null;
  }[];
};

export type TotpSetup = {
  method: "totp";
  totpURI: string;
  secret: string;
  backupCodes: string[];
};

export type SessionsResponse = {
  sessions: Session[];
  currentSessionId: string;
};

export type AppIcon = import("react").ComponentType<
  import("@x1nx3r/altics-react-icons").IconProps
>;
