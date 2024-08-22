use std::any::Any;
use std::ops::Bound;

use nu_protocol::{CustomValue, FloatRange, FromValue, IntoValue, ShellError, Span, Type, Value};
use serde::{Deserialize, Serialize};

use super::color::Color;
use super::series_2d::Series2d;

#[derive(Debug, Clone, IntoValue, Serialize, Deserialize)]
pub struct Chart2d {
    pub series: Vec<Series2d>,
    pub width: u32,
    pub height: u32,
    pub background: Option<Color>,
    pub caption: Option<String>,
    pub margin: [u32; 4], // use css shorthand rotation [top, right, bottom, left]
    pub label_area: [u32; 4],
    pub x_range: Option<Range>,
    pub y_range: Option<Range>,
}

impl Default for Chart2d {
    fn default() -> Self {
        Self {
            series: Vec::new(),
            width: 600,
            height: 400,
            background: None,
            caption: None,
            margin: [5, 5, 5, 5],
            label_area: [0, 0, 35, 35],
            x_range: None,
            y_range: None,
        }
    }
}

impl FromValue for Chart2d {
    fn from_value(v: Value) -> Result<Self, ShellError> {
        let span = v.span();
        let v = v.into_custom_value()?;
        match v.as_any().downcast_ref::<Self>() {
            Some(v) => Ok(v.clone()),
            None => {
                return Err(ShellError::CantConvert {
                    to_type: Self::ty().to_string(),
                    from_type: v.type_name(),
                    span,
                    help: None,
                })
            }
        }
    }

    fn expected_type() -> Type {
        Self::ty()
    }
}

#[typetag::serde]
impl CustomValue for Chart2d {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        Self::ty().to_string()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        Ok(self.clone().into_value(span))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

macro_rules! xy_range {
    ($fn_name:ident) => {
        pub fn $fn_name(&self) -> Option<Range> {
            if let Some(range) = self.$fn_name {
                return Some(range);
            }

            let first = self.series.first()?;
            let Range { mut min, mut max } = first.$fn_name()?;
            for Range { min: s_min, max: s_max } in self.series.iter().filter_map(|s| s.$fn_name()) {
                if s_min < min {
                    min = s_min
                }
                if s_max > max {
                    max = s_max
                }
            }

            Some(Range { min, max })
        }
    };
}

impl Chart2d {
    xy_range!(x_range);

    xy_range!(y_range);

    pub fn ty() -> Type {
        Type::Custom("plotters::chart-2d".to_string().into_boxed_str())
    }
}

#[derive(Debug, Clone, Copy, IntoValue, Serialize, Deserialize)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl FromValue for Range {
    fn from_value(v: Value) -> Result<Self, ShellError> {
        match v {
            Value::Range { val, internal_span } => {
                let range = FloatRange::from(*val);
                let min = range.start();
                let max = match range.end() {
                    Bound::Included(max) => max,
                    Bound::Excluded(max) => max,
                    Bound::Unbounded => return Err(ShellError::CantConvert { 
                        to_type: Self::expected_type().to_string(), 
                        from_type: Type::Range.to_string(), 
                        span: internal_span, 
                        help: Some("Try a bounded range instead.".to_string())
                    }),
                };

                Ok(Self { min, max })
            },

            v @ Value::List { .. } => {
                let [min, max] = <[f64; 2]>::from_value(v)?;
                Ok(Self { min, max })
            },

            v @ Value::Record { .. } => {
                #[derive(Debug, FromValue)]
                struct RangeDTO {
                    min: f64,
                    max: f64,
                }

                let RangeDTO { min, max } = RangeDTO::from_value(v)?;
                Ok(Self { min, max }) 
            },
            
            v => Err(ShellError::CantConvert {
                to_type: Self::expected_type().to_string(),
                from_type: v.get_type().to_string(),
                span: v.span(),
                help: None,
            }),
        }
    }

    fn expected_type() -> Type {
        Type::List(Box::new(Type::Number))
    }
}
