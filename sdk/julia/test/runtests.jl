using Microsandbox
using Test

@testset "Microsandbox.jl" begin
    # Test greet function (restored)
    begin
        result = greet("Test")
        @test isa(result, String)
        @test contains(result, "Test!") # Check for the name and the exclamation mark
        @test contains(result, "Welcome to Microsandbox")
    end
end
