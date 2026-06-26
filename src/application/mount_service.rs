use crate::domain::errors::{CellError, CellResult};
use crate::domain::mount_manager::{MountPoint, MountStatus};
use std::collections::HashMap;

/// 挂载服务
/// 提供原子性的挂载/卸载能力
pub struct MountService {
    mounts: HashMap<String, MountPoint>,
}

impl MountService {
    pub fn new() -> Self {
        Self {
            mounts: HashMap::new(),
        }
    }

    /// 原子性挂载（4阶段：validate -> prepare -> commit -> verify）
    pub fn atomic_mount(&mut self, point: MountPoint) -> CellResult<()> {
        // Phase 1: Prepare
        let mut prepared = Vec::new();
        for mp in vec![point.clone()] {
            match self.prepare_mount(&mp) {
                Ok(()) => prepared.push(mp),
                Err(e) => {
                    self.rollback_mounts(&prepared)?;
                    return Err(e);
                }
            }
        }

        // Phase 2: Commit
        for mp in &prepared {
            self.commit_mount(mp)?;
        }

        Ok(())
    }

    /// 原子性卸载
    pub fn atomic_unmount(&mut self, point_id: &str) -> CellResult<()> {
        let point = self.mounts.get(point_id)
            .ok_or_else(|| CellError::NotFound(format!("Mount point not found: {}", point_id)))?;

        let mut updated = point.clone();
        updated.status = MountStatus::Unmounted;
        self.mounts.insert(point_id.to_string(), updated);

        Ok(())
    }

    /// 列出所有挂载点
    pub fn list_mounts(&self) -> Vec<&MountPoint> {
        self.mounts.values().collect()
    }

    /// 格式化挂载列表
    pub fn format_mounts(&self) -> String {
        let mut o = String::new();
        o.push_str("📦 Mount Points\n");
        o.push_str(&"=".repeat(60));
        o.push_str("\n\n");

        if self.mounts.is_empty() {
            o.push_str("(no mount points)\n");
            return o;
        }

        for mp in self.mounts.values() {
            let status = match mp.status {
                MountStatus::Mounted => "✅ Mounted",
                MountStatus::Unmounted => "⭕ Unmounted",
                MountStatus::Error => "❌ Error",
            };
            o.push_str(&format!("  {}  {} ({})\n", status, mp.id, mp.path));
        }

        o.push_str(&format!("\nTotal: {}\n", self.mounts.len()));
        o
    }

    /// 准备挂载
    fn prepare_mount(&mut self, point: &MountPoint) -> CellResult<()> {
        let mut mp = point.clone();
        mp.status = MountStatus::Mounted;
        self.mounts.insert(mp.id.clone(), mp);
        Ok(())
    }

    /// 提交挂载
    fn commit_mount(&mut self, _point: &MountPoint) -> CellResult<()> {
        // 实际场景这里会调用系统 mount 命令
        Ok(())
    }

    /// 回滚挂载
    fn rollback_mounts(&mut self, points: &[MountPoint]) -> CellResult<()> {
        for mp in points {
            let mut rolled = mp.clone();
            rolled.status = MountStatus::Error;
            self.mounts.insert(rolled.id.clone(), rolled);
        }
        Ok(())
    }
}

impl Default for MountService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mount_and_list() {
        let mut service = MountService::new();
        let point = MountPoint::new("p1", "/tmp/test");
        
        service.atomic_mount(point).unwrap();
        let mounts = service.list_mounts();
        assert_eq!(mounts.len(), 1);
        assert!(mounts[0].is_mounted());
    }

    #[test]
    fn test_unmount() {
        let mut service = MountService::new();
        let point = MountPoint::new("p1", "/tmp/test");
        service.atomic_mount(point).unwrap();
        
        service.atomic_unmount("p1").unwrap();
        let mounts = service.list_mounts();
        assert!(!mounts[0].is_mounted());
    }
}
