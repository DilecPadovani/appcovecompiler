class AS3():
    '''
    Some notes for future reference:
    - At this time, missing object keys are treated as None
    - What options would we want for treating them as Undefined?
    - ValidationError support is sorely missing


    '''
    @classmethod
    def Annotate(cls, fun):
        # Convert annotations to AS3 objects
        for k, v in fun.__annotations__.items():
            fun.__annotations__[k] = cls(v, StructPath=(fun.__name__, k))

        # Identify all paramter names
        sig = inspect.signature(fun)

        @functools.wraps(fun)
        def wrapper(*args, **kwargs):
            bound = sig.bind(*args, **kwargs)
            for arg, as3 in fun.__annotations__.items():
                if arg == 'return':
                    continue
                bound.arguments[arg] = as3(bound.arguments[arg])

            rval = fun(*bound.args, **bound.kwargs)

            if 'return' in fun.__annotations__:
                rval = fun.__annotations__['return'](rval)

            return rval

        return wrapper

    __slots__ = ('CompiledFunction', 'CompiledFunctionCode',
                 'Struct', 'StructPath')

    class CompiledCodeError(Exception):
        pass

    PythonOpt = collections.namedtuple('PythonOpt', (
        'FunctionName',
        'InputVar',
        'OutputVar',
        'ErrorVar',
    ))

    def __init__(self, Struct, *, StructPath=None, Compile=True):
        if StructPath is None:
            self.StructPath = ('<Data>',)
        else:
            self.StructPath = tuple(StructPath)

        self.CompiledFunction = None
        self.CompiledFunctionCode = None

        self.Struct = self.Struct_(self.StructPath, Struct)

        if Compile:
            self.Compile()

    def __call__(self, Data):
        if not self.CompiledFunction:
            self.Compile()
        try:
            return self.CompiledFunction(Data)
        except Exception as e:
            tb = traceback.format_exc(limit=-1)
            raise self.CompiledCodeError(
                f'An error occured in the following code:\n\n' +
                ''.join(f'{i:4n}: {l}' for i, l in enumerate(self.CompiledFunctionCode.splitlines(keepends=True), 1)) + '\n\n' +
                str(Data) + '\n\n' +
                tb
            )

    def Compile(self):
        try:
            self.CompiledFunctionCode = '\n'.join(
                self.Python(FunctionName='AS3_Generated_Function'))
            l = {}
            exec(self.CompiledFunctionCode, {}, l)
            self.CompiledFunction = l['AS3_Generated_Function']
        except:
            raise

    def Python(self,
               FunctionName=None,
               InputVar='data',
               OutputVar='rval',
               ErrorVar='errs',
               Prefix='',
               ):
        Opt = self.PythonOpt(
            FunctionName=FunctionName,
            InputVar=InputVar,
            OutputVar=OutputVar,
            ErrorVar=ErrorVar,
        )

        Lines = []
        VarDepth = 0

        if Opt.FunctionName:
            Lines.append(Prefix + f'def {Opt.FunctionName}({Opt.InputVar}):')
            Prefix += '  '
            Lines.append(Prefix + f'import collections, re')
            Lines.append(Prefix + f'from Granite import aadict, Undefined')

        Lines.append(Prefix + f'{Opt.ErrorVar} = []')

        Lines.append(Prefix + f'vi{VarDepth} = {Opt.InputVar}')
        Lines.append(Prefix + f'vo{VarDepth} = Undefined')
        self.Python_(self.StructPath, self.Struct,
                     VarDepth, Prefix, Lines, Opt)
        Lines.append(Prefix + f'if {Opt.ErrorVar}:')
        Lines.append(Prefix + f'  raise Exception(str({Opt.ErrorVar}))')

        Lines.append(Prefix + f'{Opt.OutputVar} = vo{VarDepth}')

        if Opt.FunctionName:
            Lines.append(Prefix + f'return {Opt.OutputVar}')
            Prefix = Prefix[:-2]

        Lines.append(Prefix)

        return Lines

    deStruct_(self, StructPath, StructIn):
        if isinstance(StructIn, str):
            if StructIn.endswith('?'):
                StructIn = {'+Type': StructIn.removesuffix('?'), '+None': True}
            else:
                StructIn = {'+Type': StructIn}

        # copy it so we can pop keys off of it so we know if any are remaining (error)
        StructIn = dict(StructIn)
        Struct = {}

        try:
            Struct['+Source'] = StructIn.pop('+Source', None)
            Struct['+Type'] = StructIn.pop('+Type')
            if Struct['+Type'].endswith('?'):
                Struct['+Type'] = Struct['+Type'].removesuffix('?')
                Struct['+None'] = True
                if '+None' in StructIn:
                    raise TypeError(
                        f'`+Type` ended with `?` yet `+None` was specified anyway at `{"/".join(StructPath)}`')
            else:
                Struct['+None'] = bool(StructIn.pop('+None', False))

            Struct['+Label'] = StructIn.pop('+Label', StructPath[-1])
            Struct['+Help'] = StructIn.pop('+Help', None)
        except KeyError as e:
            raise TypeError(
                f'KeyError at `{"/".join(StructPath)}`: {e}') from None

        if hasattr(self, fn := f'Struct_{Struct["+Type"]}'):
            getattr(self, fn)(StructPath, StructIn, Struct)
        else:
            raise TypeError(
                f'Unrecognized type `{Struct["+Type"]}` at `{"/".join(StructPath)}`')

        # CRITICAL to clone this deeply so that we don't get shared values used as defaults
        Struct['+Default'] = copy.deepcopy(StructIn.pop('+Default', None))

        if extra := set(StructIn) - set(Struct):
            raise TypeError(
                f'Unrecognized attributes for type `{Struct["+Type"]}` at `{"/".join(StructPath)}`: {", ".join(extra)}')

        return Struct

    def Python_(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt, *, KeyVar=None, ValueVar=None):
        if not hasattr(self, fn := f'Python_{Struct["+Type"]}'):
            raise TypeError(
                f'No method `{fn}` to generate Python code for type `{Struct["+Type"]}` at `{"/".join(StructPath)}`')

        Lines.append(Prefix + f'# START {"/".join(StructPath)}')

        Lines.append(Prefix + f'try:')

        if Struct['+Default'] is not None:
            Lines.append(Prefix + f'  if vi{VarDepth} is None:')
            Lines.append(
                Prefix + f'    vi{VarDepth} = ' + repr(Struct['+Default']))
            getattr(self, fn)(StructPath, Struct,
                              VarDepth, Prefix + '  ', Lines, Opt)
        else:
            Lines.append(Prefix + f'  if vi{VarDepth} is None:')
            if Struct['+None']:
                Lines.append(Prefix + f'    vo{VarDepth} = None')
            else:
                Lines.append(
                    Prefix + f'    raise ValueError("Value must not be None")')
            Lines.append(Prefix + f'  else:')
            getattr(self, fn)(StructPath, Struct,
                              VarDepth, Prefix + '    ', Lines, Opt)

        Lines.append(Prefix + f'except (ValueError, TypeError) as e:')
        Lines.append(
            Prefix + f'  {Opt.ErrorVar}.append(({repr("/".join(StructPath))}, str(e), {KeyVar}, {ValueVar}))')

        Lines.append(Prefix + f'# END {"/".join(StructPath)}')

    def Struct_Type(self, StructPath, StructIn, Struct):
        pass

    def Python_Type(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        Lines.append(Prefix + f'vo{VarDepth} = vi{VarDepth}')

    def Struct_Boolean(self, StructPath, StructIn, Struct):
        pass

    def Python_Boolean(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        Lines.append(Prefix + f'vo{VarDepth} = bool(vi{VarDepth})')

    def Struct_Integer(self, StructPath, StructIn, Struct):
        Struct['+MaxValue'] = INTN(StructIn.pop('+MaxValue', None))
        Struct['+MinValue'] = INTN(StructIn.pop('+MaxValue', None))

    def Python_Integer(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        Lines.append(Prefix + f'vo{VarDepth} = int(vi{VarDepth})')

    def Struct_Decimal(self, StructPath, StructIn, Struct):
        Struct['+MaxValue'] = DECIMALN(StructIn.pop('+MaxValue', None))
        Struct['+MinValue'] = DECIMALN(StructIn.pop('+MaxValue', None))

    def Python_Decimal(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        Lines.append(Prefix + f'vo{VarDepth} = Decimal(vi{VarDepth})')

    def Struct_Float(self, StructPath, StructIn, Struct):
        Struct['+MaxValue'] = DECIMALN(StructIn.pop('+MaxValue', None))
        Struct['+MinValue'] = DECIMALN(StructIn.pop('+MaxValue', None))

    def Python_Float(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        Lines.append(Prefix + f'vo{VarDepth} = float(vi{VarDepth})')

    def Struct_Enum(self, StructPath, StructIn, Struct):
        Struct['+Values'] = StructIn.pop('+Values')

    def Python_Enum(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        Lines.append(Prefix + f'vo{VarDepth} = vi{VarDepth}')
        Lines.append(
            Prefix + f'if vo{VarDepth} not in {repr(Struct["+Values"])}:')
        Lines.append(
            Prefix + f'  raise ValueError("Value must be one of {repr(Struct["+Values"])}")')

    def Struct_String(self, StructPath, StructIn, Struct):
        Struct['+MaxLength'] = INTN(StructIn.pop('+MaxLength', None))
        Struct['+MinLength'] = INTN(StructIn.pop('+MinLength', None))
        Struct['+Strip'] = BOOLN(StructIn.pop('+Strip', True))
        Struct['+Regex'] = STRN(StructIn.pop('+Regex', None))

    def Python_String(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        Lines.append(Prefix + f'vo{VarDepth} = str(vi{VarDepth})')

        if Struct['+Strip'] is not None:
            Lines.append(Prefix + f'vo{VarDepth} = vo{VarDepth}.strip()')

        if Struct['+MinLength'] is not None:
            Lines.append(
                Prefix + f'if len(vo{VarDepth}) > {repr(Struct["+MinLength"])}:')
            Lines.append(Prefix + f'  raise ValueError("Input too short")')

        if Struct['+MaxLength'] is not None:
            Lines.append(
                Prefix + f'if len(vo{VarDepth}) > {repr(Struct["+MaxLength"])}:')
            Lines.append(Prefix + f'  raise ValueError("Input too long")')

        if Struct['+Regex'] is not None:
            Lines.append(
                Prefix + f'if not re.match({repr(Struct["+Regex"])}, vo{VarDepth}):')
            Lines.append(
                Prefix + f'  raise ValueError("Does not match regex: {Struct["+Regex"]}")')

    def Struct_Object(self, StructPath, StructIn, Struct):
        Struct['+Extra'] = BOOLN(StructIn.pop('+Extra', False))
        for k in tuple(StructIn):
            if k.startswith('+'):
                continue
            Struct[k] = self.Struct_(StructPath + (k,), StructIn.pop(k))

    def Python_Object(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        Lines.append(
            Prefix + f'if isinstance(vi{VarDepth}, collections.abc.Mapping):')
        Lines.append(Prefix + f'  vo{VarDepth} = aadict()')

        for fieldname, fieldstruct in Struct.items():
            if fieldname.startswith('+'):
                continue

            Lines.append(
                Prefix + f'  vi{VarDepth+1} = vi{VarDepth}.get({repr(fieldname)})')
            Lines.append(Prefix + f'  vo{VarDepth+1} = Undefined')
            self.Python_(StructPath + (fieldname,), fieldstruct, VarDepth+1,
                         Prefix + '  ', Lines, Opt, KeyVar=f'vi{VarDepth+1}')
            Lines.append(
                Prefix + f'  vo{VarDepth}[{repr(fieldname)}] = vo{VarDepth+1}')
            Lines.append(Prefix)

        if Struct['+Extra']:
            Lines.append(Prefix + f'  for k, v in vi{VarDepth}.items():')
            Lines.append(Prefix + f'    if k not in ' +
                         repr(tuple(Struct)) + ':')
            Lines.append(Prefix + f'      vo{VarDepth}[k] = v')

        Lines.append(Prefix + f'else:')
        Lines.append(
            Prefix + f'  raise ValueError(f"Invalid type: {{vi{VarDepth}}}")')

    def Struct_Map(self, StructPath, StructIn, Struct):
        if '+KeyType' in StructIn:
            Struct['+KeyType'] = self.Struct_(StructPath +
                                              ('+KeyType',), StructIn.pop('+KeyType'))
        else:
            raise TypeError(
                f'Missing `+KeyType` for type `{Struct["+Type"]}` at `{"/".join(StructPath)}`')

        if '+ValueType' in StructIn:
            Struct['+ValueType'] = self.Struct_(StructPath + (
                '+ValueType',), StructIn.pop('+ValueType'))
        else:
            raise TypeError(
                f'Missing `+ValueType` for type `{Struct["+Type"]}` at `{"/".join(StructPath)}`')

    def Python_Map(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):

        Lines.append(
            Prefix + f'if isinstance(vi{VarDepth}, collections.abc.Mapping):')
        Lines.append(Prefix + f'  vo{VarDepth} = {{}}')
        Lines.append(
            Prefix + f'  for vi{VarDepth+1}k, vi{VarDepth+1}v in vi{VarDepth}.items():')

        Lines.append(Prefix + f'    # Process Key')
        Lines.append(Prefix + f'    vi{VarDepth+1} = vi{VarDepth+1}k')
        Lines.append(Prefix + f'    vo{VarDepth+1} = Undefined')
        self.Python_(StructPath + ('+KeyType',),
                     Struct['+KeyType'], VarDepth+1, Prefix + '    ', Lines, Opt, KeyVar=f'vi{VarDepth+1}k')
        Lines.append(Prefix + f'    vo{VarDepth+1}k = vo{VarDepth+1}')

        Lines.append(Prefix)

        Lines.append(Prefix + f'    # Process Value')
        Lines.append(Prefix + f'    vi{VarDepth+1} = vi{VarDepth+1}v')
        Lines.append(Prefix + f'    vo{VarDepth+1} = Undefined')
        self.Python_(StructPath + ('+ValueType',), Struct['+ValueType'], VarDepth+1, Prefix +
                     '    ', Lines, Opt, KeyVar=f'vi{VarDepth+1}k', ValueVar=f'vi{VarDepth+1}v')
        Lines.append(Prefix + f'    vo{VarDepth+1}v = vo{VarDepth+1}')

        Lines.append(Prefix)

        Lines.append(
            Prefix + f'    vo{VarDepth}[vo{VarDepth+1}k] = vo{VarDepth+1}v')

        Lines.append(Prefix + f'else:')
        Lines.append(
            Prefix + f'  raise ValueError(f"Must be Mapping: {{vi{VarDepth}}}")')

        Lines.append(Prefix)

    def Struct_Set(self, StructPath, StructIn, Struct):
        if '+ValueType' in StructIn:
            Struct['+ValueType'] = self.Struct_(StructPath + (
                '+ValueType',), StructIn.pop('+ValueType'))
        else:
            raise TypeError(
                f'Missing `+ValueType` for type `{Struct["+Type"]}` at `{"/".join(StructPath)}`')

    def Python_Set(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        Lines.append(
            Prefix + f'if isinstance(vi{VarDepth}, collections.abc.Iterable):')
        Lines.append(Prefix + f'  vo{VarDepth} = set()')
        Lines.append(Prefix + f'  for vi{VarDepth+1} in vi{VarDepth}:')

        Lines.append(Prefix + f'    vo{VarDepth+1} = Undefined')
        self.Python_(StructPath + ('+ValueType',),
                     Struct['+ValueType'], VarDepth+1, Prefix + '    ', Lines, Opt, ValueVar=f'vi{VarDepth+1}')
        Lines.append(Prefix + f'    vo{VarDepth}.add(vo{VarDepth+1})')

        Lines.append(Prefix + f'else:')
        Lines.append(
            Prefix + f'  raise ValueError(f"Must be Iterable: {{vi{VarDepth}}}")')

        Lines.append(Prefix)

    def Struct_List(self, StructPath, StructIn, Struct):
        Struct['+Length'] = INTN(StructIn.pop('+Length', None))
        Struct['+MaxLength'] = INTN(StructIn.pop('+MaxLength', None))
        Struct['+MinLength'] = INTN(StructIn.pop('+MinLength', None))

        if '+ValueType' in StructIn:
            Struct['+ValueType'] = self.Struct_(StructPath + (
                '+ValueType',), StructIn.pop('+ValueType'))
        else:
            raise TypeError(
                f'Missing `+ValueType` for type `{Struct["+Type"]}` at `{"/".join(StructPath)}`')

    def Python_List(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        Lines.append(Prefix + f'if isinstance(vi{VarDepth}, str):')
        Lines.append(
            Prefix + f'  raise TypeError(f"Must be Iterable but not a string: {{vi{VarDepth}}}")')
        Lines.append(
            Prefix + f'elif isinstance(vi{VarDepth}, collections.abc.Iterable):')
        Lines.append(Prefix + f'  vo{VarDepth} = []')
        Lines.append(Prefix + f'  for vi{VarDepth+1} in vi{VarDepth}:')

        Lines.append(Prefix + f'    vo{VarDepth+1} = Undefined')
        self.Python_(StructPath + ('+ValueType',),
                     Struct['+ValueType'], VarDepth+1, Prefix + '    ', Lines, Opt, ValueVar=f'vi{VarDepth+1}')
        Lines.append(Prefix + f'    vo{VarDepth}.append(vo{VarDepth+1})')
        Lines.append(Prefix + f'  pass#for')

        if Struct['+Length'] is not None:
            Lines.append(Prefix + f'  # +Length')
            Lines.append(
                Prefix + f'  if len(vo{VarDepth}) != {repr(Struct["+Length"])}:')
            Lines.append(
                Prefix + f'    raise ValueError(f"List must contain exactly {Struct["+Length"]} items, but contains {{len(vo{VarDepth})}} items.")')

        if Struct['+MaxLength'] is not None:
            Lines.append(Prefix + f'  # +MaxLength')
            Lines.append(
                Prefix + f'  if len(vo{VarDepth}) > {repr(Struct["+MaxLength"])}:')
            Lines.append(
                Prefix + f'    raise ValueError("List must contain at most {Struct["+MaxLength"]} items.")')

        if Struct['+MinLength'] is not None:
            Lines.append(Prefix + f'  # +MinLength')
            Lines.append(
                Prefix + f'  if len(vo{VarDepth}) < {repr(Struct["+MinLength"])}:')
            Lines.append(
                Prefix + f'    raise ValueError("List must contain at least {Struct["+MinLength"]} items.")')

        Lines.append(Prefix + f'else:')
        Lines.append(
            Prefix + f'  raise TypeError(f"Must be Iterable: {{vi{VarDepth}}}")')

        Lines.append(Prefix)

    def Struct_Email(self, StructPath, StructIn, Struct):
        self.Struct_String(StructPath, StructIn, Struct)

    def Python_Email(self, StructPath, Struct, VarDepth, Prefix, Lines, Opt):
        self.Python_String(StructPath, Struct, VarDepth, Prefix, Lines, Opt)
