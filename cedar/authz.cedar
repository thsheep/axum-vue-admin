// 禁止操作预设角色
@id("Preset roles are read-only")
forbid (   
	principal,   
	action,   
	resource
) when {
  resource in [Role::"1", Role::"2", Role::"3"]
}; 


// 禁止操作预设组
@id("Preset Group are read-only")
forbid (
  principal,
  action,
  resource
) when {
  resource in [Group::"1", Group::"2"]
};


// 禁止操作预设策略
@id("Preset policies are read-only")
forbid (
  principal,
  action,
  resource
) when {
  resource in [Policy::"1", Policy::"2", Policy::"3", Policy::"4", Policy::"5", Policy::"6"]
};


// 超级管理员可以执行所有操作
@id("The SuperAdmin has full system privileges.")
permit (
    principal in Role::"1",
    action,
    resource
);


// 用户管理员
@id("User Administrator")
permit (
    principal in Role::"2",
    action in [
        Action::"ViewUser",
        Action::"CreateUser", 
        Action::"UpdateUser",
        Action::"DeleteUser",
        Action::"ViewGroup",
        Action::"CreateGroup",
        Action::"UpdateGroup",
        Action::"DeleteGroup",
        Action::"ViewRole",
        Action::"CreateRole",
        Action::"UpdateRole",
        Action::"DeleteRole",
        Action::"AssignRole",
        Action::"RevokeRole",
        Action::"ViewDepartment",
        Action::"CreateDepartment",
        Action::"UpdateDepartment",
        Action::"DeleteDepartment",
        ],
    resource
);

// 策略管理员
@id("Policy Administrator")
permit (
    principal in Role::"4",
    action in [
        Action::"ViewPolicy",
        Action::"CreatePolicy",
        Action::"UpdatePolicy",
        Action::"DeletePolicy",
    ], 
    resource
);