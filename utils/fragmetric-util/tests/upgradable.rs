use fragmetric_util::*;

#[derive(RequireUpgradable)]
pub struct Point2D {
    #[upgradable(latest = Point2DimV2, variant = V2)]
    pub data: VersionedData,
}

impl Upgradable for Point2D {
    type LatestVersion = Point2DimV2;

    fn upgrade(&mut self) {
        match &mut self.data {
            VersionedData::V1(old) => {
                self.data = VersionedData::V2(Point2DimV2 { x: old.0, y: old.1 });
            }
            VersionedData::V2(_) => (),
        }
    }
}

pub enum VersionedData {
    V1(Point2DimV1),
    V2(Point2DimV2),
}

pub struct Point2DimV1(u32, u32);
pub struct Point2DimV2 {
    x: u32,
    y: u32,
}

#[test]
fn test_version_upgrade_internally() {
    let mut outdated_point = Point2D {
        data: VersionedData::V1(Point2DimV1(3, 2)),
    };

    let latest_version_point = outdated_point.to_latest_version();
    assert_eq!(latest_version_point.x, 3);
    assert_eq!(latest_version_point.y, 2);
}
