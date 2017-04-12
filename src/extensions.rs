use std::convert::AsRef;
use std::path::Path;
use hyper::header;
use hyper::status::StatusCode;
use err::Error;
use {Request, Response};
use std::str::FromStr;

pub trait ResponseExtensions {
    
    fn set_body<T: Into<Vec<u8>>>(&mut self, vec: T) -> &mut Self;
    fn with_body<T: Into<Vec<u8>>>(self, t: T) -> Self;

    fn set_header<H: header::Header + header::HeaderFormat>(&mut self, h: H) -> &mut Self;
    fn with_header<H: header::Header + header::HeaderFormat>(self, h: H) -> Self;

    fn set_status(&mut self, s: StatusCode) -> &mut Self;
    fn with_status(self, s: StatusCode) -> Self;
    
    fn set_path<T: AsRef<Path>>(&mut self, path: T) -> Result<&mut Self, Error>;
    fn with_path<T: AsRef<::std::path::Path>>(self, path: T) -> Result<Self, Error> where Self: Sized;
}

impl ResponseExtensions for Response {

    fn set_body<T: Into<Vec<u8>>>(&mut self, t: T) -> &mut Self {
        let vec = t.into();
        self.headers.set(header::ContentLength(vec.len() as u64));
        self.body = Some(Box::new(vec));
        self
    }

    fn with_body<T: Into<Vec<u8>>>(mut self, t: T) -> Self {
        self.set_body(t);
        self
    }

    fn set_header<H: header::Header + header::HeaderFormat>(&mut self, h: H) -> &mut Self {
        self.headers.set(h);
        self
    }

    fn with_header<H: header::Header + header::HeaderFormat>(mut self, h: H) -> Self {
        self.set_header(h);
        self
    }

    fn set_status(&mut self, s: StatusCode) -> &mut Self {
        self.status = Some(s);
        self
    }


    fn with_status(mut self, s: StatusCode) -> Self  {
        self.set_status(s);
        self
    }

    fn set_path<T: AsRef<Path>>(&mut self, path: T) -> Result<&mut Self, Error> where Self: Sized {
        lazy_static! {
            pub static ref MIME_TYPES: ::conduit_mime_types::Types = 
                ::conduit_mime_types::Types::new().unwrap();
        }
        
        let path_as_ref = path.as_ref();

        let md = path_as_ref.metadata()?;

        let mime = MIME_TYPES
            .mime_for_path(path_as_ref)
            .parse::<::hyper::mime::Mime>()
            .map_err(|_| "Unknown MIME type")?;

        let file = ::std::fs::File::open(path_as_ref)?;

        self.set_header(header::ContentLength(md.len() as u64));
        self.set_header(header::ContentType(mime));

        self.body = Some(Box::new(file));

        Ok(self)
    }

    fn with_path<T: AsRef<::std::path::Path>>(mut self, path: T) -> Result<Self, Error> {
        self.set_path(path)?;
        Ok(self)
    }

}

pub trait RequestExtensions {
    fn extract_captures<T: CaptureExtraction>(&self) -> Result<T, Error>;
}

impl<'a, 'b, 'c> RequestExtensions for Request<'a, 'b, 'c> {
    fn extract_captures<T: CaptureExtraction>(&self) -> Result<T, Error> {
        Ok(T::extract_captures(self)?)
    }
}

pub trait CaptureExtraction: Sized {
    fn extract_captures(req: &Request) -> Result<Self, Error>;
}

impl<T> CaptureExtraction for (T,) where T: FromStr {
    fn extract_captures(req: &Request) -> Result<Self, Error> {
        let caps = req.captures().ok_or("No captures")?;
        let out_1 = caps.get(1).map(|x| x.as_str() ).and_then(|x| x.parse().ok())
            .ok_or("Couldn't parse capture")?;
        Ok((out_1,))
    }
}

impl<T1, T2> CaptureExtraction for (T1, T2) where T1: FromStr, T2: FromStr {
    fn extract_captures(req: &Request) -> Result<Self, Error> {
        let caps = req.captures().ok_or("No captures")?;
        let out_1 = caps.get(1).map(|x| x.as_str() ).and_then(|x| x.parse().ok())
            .ok_or("Couldn't parse capture")?;
        let out_2 = caps.get(2).map(|x| x.as_str() ).and_then(|x| x.parse().ok())
            .ok_or("Couldn't parse capture")?;
        Ok((out_1, out_2))
    }
}

impl<T1, T2, T3> CaptureExtraction for (T1, T2, T3) where T1: FromStr, T2: FromStr, T3: FromStr {
    fn extract_captures(req: &Request) -> Result<Self, Error> {
        let caps = req.captures().ok_or("No captures")?;
        let out_1 = caps.get(1).map(|x| x.as_str() ).and_then(|x| x.parse().ok())
            .ok_or("Couldn't parse capture")?;
        let out_2 = caps.get(2).map(|x| x.as_str() ).and_then(|x| x.parse().ok())
            .ok_or("Couldn't parse capture")?;
        let out_3 = caps.get(3).map(|x| x.as_str() ).and_then(|x| x.parse().ok())
            .ok_or("Couldn't parse capture")?;
        Ok((out_1, out_2, out_3))
    }
}


