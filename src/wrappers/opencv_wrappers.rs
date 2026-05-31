use opencv::core::{MatTraitConst, Size, ToInputOutputArray, UMatTraitConst};
use opencv::{
    core::{Mat, ToInputArray, ToOutputArray, UMat},
    Result as CvResult,
};

pub trait ImageBuffer: ToInputArray + ToOutputArray + ToInputOutputArray + Sized {
    fn new_empty() -> CvResult<Self>;

    fn size(&self) -> CvResult<Size>;

    #[allow(unused)] // for future reference
    fn to_gpu(self) -> CvResult<UMat>;

    fn to_cpu(self) -> CvResult<Mat>;
}

impl ImageBuffer for Mat {
    fn new_empty() -> CvResult<Self> {
        Ok(Mat::default())
    }

    fn size(&self) -> CvResult<Size> {
        MatTraitConst::size(self)
    }

    fn to_gpu(self) -> CvResult<UMat> {
        let mut gpu_frame = UMat::new_def();
        self.copy_to(&mut gpu_frame)?;
        Ok(gpu_frame)
    }

    fn to_cpu(self) -> CvResult<Mat> {
        Ok(self)
    }
}

impl ImageBuffer for UMat {
    fn new_empty() -> CvResult<Self> {
        Ok(UMat::new_def())
    }

    fn size(&self) -> CvResult<Size> {
        UMatTraitConst::size(self)
    }

    fn to_gpu(self) -> CvResult<UMat> {
        Ok(self)
    }

    fn to_cpu(self) -> CvResult<Mat> {
        let mut cpu_image = Mat::default();
        self.copy_to(&mut cpu_image)?;
        Ok(cpu_image)
    }
}
